import React from 'react';
import { render } from 'react-dom';
import { Provider } from 'react-redux';
import { withStyles } from 'material-ui/styles';
import CssBaseline from 'material-ui/CssBaseline';
import ErrorPrompt from './ErrorPrompt.jsx';
import FeatureGroup from './FeatureGroup.jsx';
import PathMenu from './PathMenu.jsx';
// import Store from './Store.jsx';

import { connector, store } from './store';
import Updater from './Updater.jsx';

const styles = theme => ({
  root: {
    flexGrow: 1,
    zIndex: 1,
    overflow: 'hidden',
    position: 'relative',
    display: 'flex',
    width: '100%',
  },
  content: {
    flexGrow: 1,
    backgroundColor: theme.palette.background.default,
    padding: theme.spacing.unit * 3,
  },
});

class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      selected: null
    }
  }

  componentDidMount() {
    this.props.loadApps()
  }

  isSelected(key) {
    return this.state.selected === key;
  }

  setSelected(key) {
    return () => {
      this.setState((prevState, props) => {
        return {
          selected: prevState.selected !== key ? key : null
        };
      })
    }
  }

  render() {
    let { classes, app, env, apps, flags } = this.props;

    return (
      <div className={classes.root}>
        <PathMenu
          menuToggle={() => {}}
          open={true} />
        <main className={classes.content}>
          <FeatureGroup />
        </main>
      </div>
    );
  }
}

// {
//   this.props.flags.map(group => {
//     let key = `${group.app}::${group.env}`;
//     let selected = this.isSelected(key);
//     let adder = key => {
//       this.props.onAdd(group.app, group.env, key);
//     };
//     let remover = key => {
//       this.props.onDelete(group.app, group.env, key);
//     };

//     return ;
//   })
// }

let StyledApp = connector(withStyles(styles)(App));

// function Run() {
//   return (
//     <ErrorPrompt>
//       <Store baseUrl="/api/v1">
//         <StyledApp />
//       </Store>
//     </ErrorPrompt>
//   );
// }

function Run() {
  return (
    <ErrorPrompt>
      <Provider store={store}>
        <div>
          <Updater />
          <StyledApp />
        </div>
      </Provider>
    </ErrorPrompt>
  );
}

render(<Run />, document.querySelector('#app'));