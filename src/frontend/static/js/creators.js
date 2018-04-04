import axios from "axios";

import * as actions from "./actions";

function t() {
  return Math.floor(Date.now() / 1000);
}

function newFlag(app, env, key) {
  return {
    app,
    env,
    key,
    value: true,
    version: 1,
    enabled: true,
    created: t(),
    updated: t()
  };
}

function auth(getState) {
  let { apiKey, apiSecret } = getState();

  return {
    Authorization: btoa(apiKey + ':' + apiSecret)
  };
}

export function addFlag(key) {
  return function(dispatch, getState) {
    let { baseUrl, app, env } = getState();
    let url = `${baseUrl}/${app}/${env}/flag/`;
    let flag = newFlag(app, env, key);

    dispatch({ type: actions.ADD_FLAG, payload: flag });

    return axios.post(url, flag, { headers: auth(getState) }).catch(err => {
      dispatch({ type: actions.DELETE_FLAG, payload: flag.key });
      // this.props.onError(`Failed to create flag ${flag.key}`);
    });
  };
}

export function deleteFlag(key) {
  return function(dispatch, getState) {
    let { baseUrl, app, env, flags } = getState();
    let url = `${baseUrl}/${app}/${env}/flag/${key}/`;
    let flag = flags.filter(f => f.key === key);

    dispatch({ type: DELETE_FLAG, payload: key });

    return axios.delete(url, { headers: auth(getState) }).catch(err => {
      if (flag.length > 0) {
        dispatch({ type: actions.ADD_FLAG, payload: flag[0] });
      }
      // this.props.onError(`Failed to delete flag ${key}`);
    });
  };
}

export function loadApps() {
  return function(dispatch, getState) {
    let { baseUrl, key, secret } = getState();
    let url = `${baseUrl}/paths/`;

    axios.get(url, { headers: auth(getState) }).then(function(resp) {
        let apps = resp.data;

        if (apps.length === 0) {
          throw "No apps available";
        }

        apps = apps.sort((a, b) => {
          if (a.path !== b.path) {
            return a.path > b.path;
          }

          return 0;
        });

        dispatch({ type: actions.LOAD_APPS, payload: apps });

        // Load the first returned app as default
        if (apps.length > 0) {
          let firstApp = apps[0];
          selectApp(firstApp.app, firstApp.env)(dispatch, getState);
        }
      }).catch(function(err) {
        dispatch({ type: actions.UNLOAD_DATA, payload: undefined });
      });
  };
}

export function loadFlags(app, env) {
  return function(dispatch, getState) {
    let { baseUrl } = getState();
    let url = `${baseUrl}/${app}/${env}/flags/`;

    axios.get(url, { headers: auth(getState) })
      .then(function(resp) {
        dispatch({ type: actions.LOAD_DATA, payload: resp.data });
      });
  };
}

export function selectApp(app, env) {
  return function(dispatch, getState) {
    dispatch({ type: actions.SELECT_APP, payload: { app, env } });
    loadFlags(app, env)(dispatch, getState);
  };
}

export function updateFlag(key, enabled) {
  return function(dispatch, getState) {
    let { baseUrl, app, env, flags } = getState();
    let url = `${baseUrl}/${app}/${env}/flag/${key}/`;
    let flag = flags.filter(f => f.key === key);

    if (flag.length > 0) {
      let f = {
        app,
        env,
        key: flag[0].key,
        value: flag[0].value,
        version: flag[0].version,
        created: flag[0].created,
        updated: t(),
        enabled
      };

      dispatch({ type: actions.UPDATE_FLAG, payload: f });

      axios.post(url, f, { headers: auth(getState) })
        .catch(err => {
          f.enabled = !f.enabled;
          dispatch({ type: actions.UPDATE_FLAG, payload: f });
        });
    }
  };
}

export function updateKey(key) {
  return function(dispatch, getState) {
    dispatch({ type: actions.UPDATE_KEY, payload: key });
    loadApps()(dispatch, getState);
  }
}

export function updateSecret(secret) {
  return function(dispatch, getState) {
    dispatch({ type: actions.UPDATE_SECRET, payload: secret });
    loadApps()(dispatch, getState);
  }
}

export function toggleMenu(state) {
  return { type: actions.TOGGLE_MENU, payload: state };
}