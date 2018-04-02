import * as actions from "./actions";
import * as creators from "./creators";
import { createStore, applyMiddleware, compose } from "redux";
import thunk from 'redux-thunk'
import { connect } from "react-redux";

let initialState = {
  baseUrl: "/api/v1",
  app: "",
  env: "",
  apps: [],
  flags: [],
  refresh: 1000
};

function reducer(state = initialState, action) {
  switch(action.type) {
    case actions.ADD_FLAG: {
      let flag = action.payload;
      let flags = state.flags.slice();
      flags.push(flag);

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

    default:
      return state;
  }
}

const mapStateToProps = state => {
  return {
    app: state.app,
    env: state.env,
    apps: state.apps,
    flags: state.flags
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
    loadFlags(app, env) {
      dispatch(creators.loadFlags(app, env));
    },
    selectApp(app, env) {
      dispatch(creators.selectApp(app, env));
    },
    updateFlag(key, enabled) {
      dispatch(creators.updateFlag(key, enabled));
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