import axios from "axios";
import debounce from "lodash.debounce";

import * as actions from "./actions";
import { get, set } from "./auth";

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
    Authorization: 'Basic ' + btoa(apiKey + ':' + apiSecret)
  };
}

function token(getState) {
  let { token } = getState();

  return {
    Authorization: 'Token ' + token
  };
}

export function addFlag(key) {
  return function(dispatch, getState) {
    let { baseUrl, app, env } = getState();
    let url = `${baseUrl}/${app}/${env}/flag/`;
    let flag = newFlag(app, env, key);

    dispatch({ type: actions.ADD_FLAG, payload: flag });

    return axios.post(url, flag, { headers: token(getState) }).catch(err => {
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

    return axios.delete(url, { headers: token(getState) }).catch(err => {
      if (flag.length > 0) {
        dispatch({ type: actions.ADD_FLAG, payload: flag[0] });
      }
      // this.props.onError(`Failed to delete flag ${key}`);
    });
  };
}

function sortApps(apps) {
  return apps.sort((a, b) => {
    return a.path.localeCompare(b.path);
  });
}

export function loadApps() {
  return debounce(
    function(dispatch, getState) {
      let { baseUrl, key, secret } = getState();
      let url = `${baseUrl}/paths/`;

      axios.get(url, { headers: token(getState) }).then(function(resp) {
          let apps = resp.data;

          sortApps(apps);

          dispatch({ type: actions.LOAD_APPS, payload: apps });

          if (apps.length > 0) {
            selectApp(apps[0].app, apps[0].env)(dispatch, getState);
          }
        }).catch(function(err) {
          dispatch({ type: actions.UNLOAD_DATA, payload: undefined });
        });
    }, 250);
}

export function loadFlags(flags) {
  return { type: actions.LOAD_DATA, payload: flags };
}

export function loadFlagsFor(app, env) {
  return function(dispatch, getState) {
    if (app && env) {
      let { baseUrl } = getState();
      let url = `${baseUrl}/${app}/${env}/flags/`;

      axios.get(url, { headers: token(getState) })
        .then(function(resp) {
          dispatch(loadFlags(resp.data));
        });
    }
  };
}

export function clearFlags() {
  return function(dispatch, getState) {
    dispatch({ type: actions.LOAD_DATA, payload: [] });
  };
}

export function selectApp(app, env) {
  return function(dispatch, getState) {
    dispatch({ type: actions.SELECT_APP, payload: { app, env } });
    loadFlagsFor(app, env)(dispatch, getState);
  };
}

export function clearApp() {
  return function(dispatch, getState) {
    dispatch({ type: actions.SELECT_APP, payload: { app: "", env: "" } });
    clearFlags()(dispatch, getState);
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

      axios.post(url, f, { headers: token(getState) })
        .catch(err => {
          f.enabled = !f.enabled;
          dispatch({ type: actions.UPDATE_FLAG, payload: f });
        });
    }
  };
}

function authenticate() {
  return debounce(
    function(dispatch, getState) {
      let { baseUrl, key, secret } = getState();
      let url = `${baseUrl}/authenticate/`;

      axios.post(url, {}, { headers: auth(getState) }).then(function(resp) {
          let token = resp.data;
          set(token);
          dispatch({ type: actions.UPDATE_TOKEN, payload: token });
          loadApps()(dispatch, getState)
        }).catch(function(err) {
          dispatch({ type: actions.UNLOAD_DATA, payload: undefined });
        });
    }, 250);
}

export function updateKey(key) {
  return function(dispatch, getState) {
    dispatch({ type: actions.UPDATE_KEY, payload: key });
    authenticate()(dispatch, getState);
  }
}

export function updateSecret(secret) {
  return function(dispatch, getState) {
    dispatch({ type: actions.UPDATE_SECRET, payload: secret });
    authenticate()(dispatch, getState);
  }
}

export function logout() {
  return function(dispatch, getState) {
    set('');
    dispatch({ type: actions.UNLOAD_DATA, payload: undefined });
    dispatch({ type: actions.LOGOUT, payload: undefined });
  }
}

export function toggleMenu(state) {
  return { type: actions.TOGGLE_MENU, payload: state };
}

export function updateFilter(history, filterText) {
  return function (dispatch, getState) {
    dispatch({ type: actions.UPDATE_FILTER, payload: filterText });
    history.replace(history.location.pathname + (filterText ? `?s=${filterText}` : ''));
  };
}

export function toggleAppCreate(state) {
  return { type: actions.TOGGLE_APP_CREATE, payload: state }
}

export function addApp(app, env) {
  return function(dispatch, getState) {
    let { baseUrl } = getState();
    let url = `${baseUrl}/path/`;
    let a = { app, env };

    dispatch(toggleAppCreate(false));

    axios.post(url, a, { headers: token(getState) })
      .catch(err => {})
      .then(() => loadApps()(dispatch, getState));
  };
}

function isAppDisplayUrl(path) {
  let parts = path.split('/');
  return parts.length === 4 && parts[0] === "" && parts[3] === "";
}

function getAppParts(path) {
  let parts = path.split('/');

  if (parts.length === 4) {
    return { app: parts[1], env: parts[2] };
  }

  return null;
}

function hasFilterTerm(query) {
  return query.substr(1, 2) === "s=";
}

function getFilterTerm(query) {
  return query.substr(3);
}

export function navigate(history, location) {
  return function(dispatch, getState) {
    if (isAppDisplayUrl(location.pathname)) {
      let { app, env } = getAppParts(location.pathname);

      if (getState().app !== app || getState().env !== env) {
        selectApp(app, env)(dispatch, getState);
      }
    } else {
      clearApp()(dispatch, getState);
    }

    if (hasFilterTerm(location.search)) {
      let term = getFilterTerm(location.search);

      if (getState().filterText !== term) {
        updateFilter(history, term)(dispatch, getState);
      }
    } else {
      if (getState().filterText !== "") {
        updateFilter(history, "")(dispatch, getState);
      }
    }
  }
}