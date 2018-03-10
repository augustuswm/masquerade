use hyper;
use rusoto_core::region::Region;
use rusoto_core::request::{default_tls_client, DispatchSignedRequest, TlsError};
use rusoto_credential::{AwsCredentials, BaseAutoRefreshingProvider, ChainProvider,
                        CredentialsError, DefaultCredentialsProviderSync, ProvideAwsCredentials};
use rusoto_dynamodb::{AttributeValue, DeleteItemError, DeleteItemInput, DynamoDb, DynamoDbClient,
                      GetItemError, GetItemInput, PutItemError, PutItemInput, QueryError,
                      QueryInput};

use std::fmt::Debug;
use std::collections::HashMap;
use std::time::Duration;

use error::BannerError;
use hash_cache::HashCache;
use store::Store;

pub struct DynamoStore<T, P, D>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    cache: HashCache<T>,
    client: DynamoDbClient<P, D>,
    table: String,
}

#[derive(Debug, PartialEq)]
pub enum DynamoError {
    Credentials(CredentialsError),
    Delete(DeleteItemError),
    FailedToParseResponse,
    Get(GetItemError),
    Put(PutItemError),
    Query(QueryError),
    Tls(TlsError),
}

type DefaultP = BaseAutoRefreshingProvider<ChainProvider, ::std::sync::Mutex<AwsCredentials>>;
type DefaultD = hyper::client::Client;
type DefaultDynamoStore<T> = DynamoStore<T, DefaultP, DefaultD>;

impl<T> DefaultDynamoStore<T> {
    pub fn new<S>(table: S) -> Result<DefaultDynamoStore<T>, DynamoError>
    where
        S: Into<String>,
    {
        let credentials = DefaultCredentialsProviderSync::new().map_err(DynamoError::Credentials)?;
        let client = DynamoDbClient::new(
            default_tls_client().map_err(DynamoError::Tls)?,
            credentials,
            Region::UsEast1,
        );

        Ok(DynamoStore::new_with_db(table, client))
    }
}

impl<T, P, D> DynamoStore<T, P, D>
where
    P: ProvideAwsCredentials,
    D: DispatchSignedRequest,
{
    pub fn new_with_db<S: Into<String>>(
        table: S,
        client: DynamoDbClient<P, D>,
    ) -> DynamoStore<T, P, D> {
        DynamoStore {
            cache: HashCache::new(Duration::new(0, 0)),
            client: client,
            table: table.into(),
        }
    }
}

pub trait FromAttrMap<T> {
    type Error: Debug;
    fn from_attr_map(map: HashMap<String, AttributeValue>) -> Result<T, Self::Error>;
}

impl<T, P, Provide, Dispatch> Store<P, T> for DynamoStore<T, Provide, Dispatch>
where
    P: AsRef<str>,
    T: Clone + FromAttrMap<T> + Into<HashMap<String, AttributeValue>>,
    Provide: ProvideAwsCredentials,
    Dispatch: DispatchSignedRequest,
{
    type Error = BannerError;

    fn get(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        let composite = [path.as_ref(), "$", key].concat();

        let mut key_map: HashMap<String, AttributeValue> = HashMap::new();
        let mut attr = AttributeValue::default();
        attr.s = Some(composite);
        key_map.insert("key".into(), attr);

        let mut get = GetItemInput::default();
        get.key = key_map;
        get.table_name = self.table.clone();

        let response = self.client.get_item(&get).map_err(DynamoError::Get)?.item;

        Ok(response.and_then(|mut map| {
            map.remove("data").and_then(|data| match data.m {
                Some(data_map) => T::from_attr_map(data_map).ok(),
                None => None,
            })
        }))
    }

    fn get_all(&self, path: &P) -> Result<HashMap<String, T>, BannerError> {
        let mut path_attr: HashMap<String, AttributeValue> = HashMap::new();
        let mut attr = AttributeValue::default();
        attr.s = Some(path.as_ref().to_string());
        path_attr.insert(":key_path".into(), attr);

        let mut query = QueryInput::default();
        query.index_name = Some("key_path-index".into());
        query.key_condition_expression = Some("key_path = :key_path".into());
        query.expression_attribute_values = Some(path_attr);
        query.table_name = self.table.clone();

        let response = self.client.query(&query).map_err(DynamoError::Query)?.items;

        let mut ts: HashMap<String, T> = HashMap::new();

        if let Some(rs) = response {
            let pref = [path.as_ref(), "$"].concat();

            for mut r in rs.into_iter() {
                match r.remove("key") {
                    Some(key_attr) => {
                        if let Some(ref key) = key_attr.s {
                            let t_res = r.remove("data").and_then(|data| match data.m {
                                Some(data_map) => T::from_attr_map(data_map).ok(),
                                None => None,
                            });

                            if let Some(t) = t_res {
                                let hash_k = key.trim_left_matches(pref.as_str()).to_string();
                                ts.insert(hash_k, t);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        Ok(ts)
    }

    fn delete(&self, path: &P, key: &str) -> Result<Option<T>, BannerError> {
        let composite = [path.as_ref(), "$", key].concat();

        let mut key_map: HashMap<String, AttributeValue> = HashMap::new();
        let mut attr = AttributeValue::default();
        attr.s = Some(composite);
        key_map.insert("key".into(), attr);

        let mut del = DeleteItemInput::default();
        del.return_values = Some("ALL_OLD".to_string());
        del.key = key_map;
        del.table_name = self.table.clone();

        let response = self.client
            .delete_item(&del)
            .map_err(DynamoError::Delete)?
            .attributes;

        Ok(response.and_then(|mut map| {
            map.remove("data").and_then(|data| match data.m {
                Some(data_map) => T::from_attr_map(data_map).ok(),
                None => None,
            })
        }))
    }

    fn upsert(&self, path: &P, key: &str, item: &T) -> Result<Option<T>, BannerError> {
        let composite = [path.as_ref(), "$", key].concat();

        let mut key_attr = AttributeValue::default();
        key_attr.s = Some(composite);

        let mut path_attr = AttributeValue::default();
        path_attr.s = Some(path.as_ref().to_string());

        let data: HashMap<String, AttributeValue> = item.clone().into();
        let mut data_attr = AttributeValue::default();
        data_attr.m = Some(data);

        let mut doc: HashMap<String, AttributeValue> = HashMap::new();
        doc.insert("key".into(), key_attr);
        doc.insert("key_path".into(), path_attr);
        doc.insert("data".into(), data_attr);

        let mut put = PutItemInput::default();
        put.return_values = Some("ALL_OLD".to_string());
        put.item = doc;
        put.table_name = self.table.clone();

        ;

        let response = self.client
            .put_item(&put)
            .map_err(DynamoError::Put)?
            .attributes;

        Ok(response.and_then(|mut map| {
            map.remove("data").and_then(|data| match data.m {
                Some(data_map) => T::from_attr_map(data_map).ok(),
                None => None,
            })
        }))
    }
}

#[cfg(test)]
mod tests {
    use flag::*;
    use store::*;

    use super::*;

    const PATH: &'static str = "app$";

    fn f<S: Into<String>>(key: S, enabled: bool) -> Flag {
        Flag::new(key, FlagValue::Bool(true), 1, enabled)
    }

    fn path(test: &str) -> FlagPath {
        [PATH, test].concat().parse::<FlagPath>().unwrap()
    }

    fn dataset(p: &str, dur: u64) -> DefaultDynamoStore<Flag> {
        DynamoStore::new("test").unwrap()
    }

    #[test]
    fn test_gets_items() {
        let data = dataset("get_items", 0);
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.iter() {
            data.upsert(&path("gets"), flag.key(), &flag);
        }

        assert_eq!(data.get(&path("gets"), "f1").unwrap().unwrap(), flags[0]);
        assert_eq!(data.get(&path("gets"), "f2").unwrap().unwrap(), flags[1]);
        assert!(data.get(&path("gets"), "f3").unwrap().is_none());
    }

    #[test]
    fn test_gets_all_items() {
        let flags = vec![f("f1", false), f("f2", true)];

        let data = dataset("all_items", 0);
        for flag in flags.iter() {
            data.upsert(&path("gets_all"), flag.key(), &flag);
        }

        let res = data.get_all(&path("gets_all"));

        assert!(res.is_ok());

        let map = res.unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("f1").unwrap(), &flags[0]);
        assert_eq!(map.get("f2").unwrap(), &flags[1]);
    }

    #[test]
    fn test_deletes_without_cache() {
        let data = dataset("delete_no_cache", 0);
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.iter() {
            data.upsert(&path("deletes"), flag.key(), &flag);
        }

        assert_eq!(data.get_all(&path("deletes")).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(&path("deletes"), "f1");
        assert_eq!(f1.unwrap().unwrap(), flags[0]);

        let f1_2 = data.get(&path("deletes"), "f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(&path("deletes"), "f2");
        assert_eq!(f2.unwrap().unwrap(), flags[1]);

        let f2_2 = data.get(&path("deletes"), "f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all(&path("deletes")).unwrap().len(), 0);
    }

    #[test]
    fn test_deletes_with_cache() {
        let data = dataset("delete_cache", 30);
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.iter() {
            data.upsert(&path("deletes"), flag.key(), &flag);
        }

        assert_eq!(data.get_all(&path("deletes")).unwrap().len(), 2);

        // Test flag #1
        let f1 = data.delete(&path("deletes"), "f1");
        assert_eq!(f1.unwrap().unwrap(), flags[0]);

        let f1_2 = data.get(&path("deletes"), "f1");
        assert!(f1_2.unwrap().is_none());

        // Test flag #2
        let f2 = data.delete(&path("deletes"), "f2");
        assert_eq!(f2.unwrap().unwrap(), flags[1]);

        let f2_2 = data.get(&path("deletes"), "f2");
        assert!(f2_2.unwrap().is_none());

        assert_eq!(data.get_all(&path("deletes")).unwrap().len(), 0);
    }

    #[test]
    fn test_replacements_without_cache() {
        let data = dataset("replace_no_cache", 0);
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.iter() {
            data.upsert(&path("replacements"), flag.key(), &flag);
        }

        assert_eq!(data.get_all(&path("replacements")).unwrap().len(), 2);

        // Test flag #1
        let new_f1 = f("f1", true);
        let f1 = data.upsert(&path("replacements"), "f1", &new_f1);
        assert_eq!(f1.unwrap().unwrap(), flags[0]);

        let f1_2 = data.get(&path("replacements"), "f1");
        assert_eq!(f1_2.unwrap().unwrap(), new_f1);

        assert_eq!(data.get_all(&path("replacements")).unwrap().len(), 2);
    }

    #[test]
    fn test_replacements_with_cache() {
        let data = dataset("replace_cache", 30);
        let flags = vec![f("f1", false), f("f2", true)];

        for flag in flags.iter() {
            data.upsert(&path("replacements"), flag.key(), &flag);
        }

        assert_eq!(data.get_all(&path("replacements")).unwrap().len(), 2);

        // Test flag #1
        let new_f1 = f("f1", true);
        let f1 = data.upsert(&path("replacements"), "f1", &new_f1);
        assert_eq!(f1.unwrap().unwrap(), flags[0]);

        let f1_2 = data.get(&path("replacements"), "f1");
        assert_eq!(f1_2.unwrap().unwrap(), new_f1);

        assert_eq!(data.get_all(&path("replacements")).unwrap().len(), 2);
    }
}
