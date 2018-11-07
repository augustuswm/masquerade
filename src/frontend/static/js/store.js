import { createStore, applyMiddleware, compose } from "redux";
import thunk from 'redux-thunk'
import { connect } from "react-redux";

import * as actions from "./actions";
import * as creators from "./creators";
import { get, set } from "./auth";

let loadedPath = window.location.pathname.split('/', 3);
let loadedApp = loadedPath[1];
let loadedEnv = loadedPath[2];

let initialState = {
  baseUrl: "/api/v1",
  app: loadedApp || "",
  env: loadedEnv || "",
  apps: [],
  flags: [],
  refresh: 1000,
  apiKey: "",
  apiSecret: "",
  token: get() || "",
  pathMenuOpen: false,
  filterText: "",
  appCreateModalOpen: false
};

function reducer(state = initialState, action) {
  switch(action.type) {
    case actions.ADD_FLAG: {
      let flag = action.payload;
      let flags = state.flags.slice();
      flags.push(flag);
      flags = flags.sort((a, b) => a.key > b.key);

      return Object.assign(
        {},
        state,
        { flags }
      );
    }

    case actions.DELETE_FLAG: {
      let key = action.payload;
      let flags = state.flags.filter(f => f.key !== key);

      return Object.assign(
        {},
        state,
        { flags }
      );
    }

    case actions.LOAD_APPS: {
      let apps = action.payload;

      return Object.assign(
        {},
        state,
        { apps }
      );
    }

    case actions.LOAD_DATA: {
      let flags = action.payload;

      return Object.assign(
        {},
        state,
        { flags }
      );
    }

    case actions.SELECT_APP: {
      let { app, env } = action.payload;

      return Object.assign(
        {},
        state,
        { app, env }
      );
    }

    case actions.UPDATE_FLAG: {
      let flag = action.payload;
      let flags = state.flags.slice();
      let flagIndex = flags.findIndex(f => f.key === flag.key);
      flags[flagIndex] = flag;

      return Object.assign(
        {},
        state,
        { flags }
      );
    }

    case actions.UPDATE_KEY: {
      return Object.assign(
        {},
        state,
        { apiKey: action.payload }
      );
    }

    case actions.UPDATE_SECRET: {
      return Object.assign(
        {},
        state,
        { apiSecret: action.payload }
      );
    }

    case actions.UPDATE_TOKEN: {
      return Object.assign(
        {},
        state,
        { token: action.payload }
      );
    }

    case actions.UNLOAD_DATA: {
      return Object.assign(
        {},
        state,
        { apps: [], flags: [] }
      );
    }

    case actions.LOGOUT: {
      return Object.assign(
        {},
        state,
        { apiKey: "", apiSecret:"" }
      );
    }    

    case actions.TOGGLE_MENU: {
      return Object.assign(
        {},
        state,
        { pathMenuOpen: action.payload }
      );
    }

    case actions.UPDATE_FILTER: {
      return Object.assign(
        {},
        state,
        { filterText: action.payload }
      );
    }

    case actions.TOGGLE_APP_CREATE: {
      return Object.assign(
        {},
        state,
        { appCreateModalOpen: action.payload }
      );
    }

    default:
      return state;
  }
}

const mapStateToProps = state => {
  return {
    baseUrl: state.baseUrl,
    app: state.app,
    env: state.env,
    apps: state.apps,
    flags: state.flags,
    refresh: state.refresh,
    apiKey: state.apiKey,
    apiSecret: state.apiSecret,
    pathMenuOpen: state.pathMenuOpen,
    filterText: state.filterText,
    appCreateModalOpen: state.appCreateModalOpen
  };
};

const mapDispatchToProps = dispatch => {
  return {
    addFlag(key) {
      dispatch(creators.addFlag(key));
    },
    deleteFlag(key) {
      dispatch(creators.deleteFlag(key));
    },
    loadApps() {
      dispatch(creators.loadApps());
    },
    loadFlags(flags) {
      dispatch(creators.loadFlags(flags));
    },
    loadFlagsFor(app, env) {
      dispatch(creators.loadFlagsFor(app, env));
    },
    selectApp(app, env) {
      dispatch(creators.selectApp(app, env));
    },
    updateFlag(key, enabled) {
      dispatch(creators.updateFlag(key, enabled));
    },
    updateKey(key) {
      dispatch(creators.updateKey(key));
    },
    updateSecret(secret) {
      dispatch(creators.updateSecret(secret));
    },
    logout() {
      dispatch(creators.logout());
    },
    toggleMenu(state) {
      dispatch(creators.toggleMenu(state));
    },
    updateFilter(history, filterText) {
      dispatch(creators.updateFilter(history, filterText));
    },
    toggleAppCreate(state) {
      dispatch(creators.toggleAppCreate(state));
    },
    addApp(app, env) {
      dispatch(creators.addApp(app, env));
    },
    bindHistory(history) {
      history.listen(location => dispatch(creators.navigate(history, location)));
    }
  };
};

const composer = typeof window === 'object' && window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__ ?
  window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__({}) : compose;

export const store = createStore(
  reducer,
  composer(applyMiddleware(thunk))
);
export const connector = connect(
  mapStateToProps,
  mapDispatchToProps
);