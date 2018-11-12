import React from 'react';
import { render } from 'react-dom';
import { Provider } from 'react-redux';
import { BrowserRouter, withRouter } from 'react-router-dom'

import { withStyles } from 'material-ui/styles';
import CssBaseline from 'material-ui/CssBaseline';
import { createMuiTheme, MuiThemeProvider } from 'material-ui/styles';
import Hidden from 'material-ui/Hidden';
import Typography from 'material-ui/Typography';

import { connector, store } from './store';
import ErrorPrompt from './ErrorPrompt.jsx';
import FeatureGroup from './FeatureGroup.jsx';
import PathMenu from './PathMenu.jsx';
import Login from './Login.jsx';
import Updater from './Updater.jsx';
import CreateMenu from './CreateMenu.jsx';
import Header from './Header.jsx';
import FeaturePanels from './FeaturePanels.jsx';
import CreateApp from './CreateApp.jsx';

const theme = createMuiTheme({
  palette: {
    primary: {
      light: '#62d493',
      main: '#28A265',
      dark: '#00723a',
      contrastText: '#fff',
    },
    secondary: {
      light: '#fda0f3',
      main: '#c970c0',
      dark: '#97418f',
      contrastText: '#fff',
    }
  }
});

const Fragment = React.Fragment;

const styles = theme => ({
  root: {
    flexGrow: 1,
    zIndex: 1,
    overflow: 'hidden',
    position: 'relative',
    display: 'flex',
    flexDirection: 'column',
    width: '100%',
    opacity: 1,
    transition: 'opacity',
    transitionDuration: '0.25s',
    transitionDelay: '1s',
    height: '100vh'
  },
  content: {
    flexGrow: 1,
    backgroundColor: theme.palette.background.default,
    padding: theme.spacing.unit * 3,
    overflowY: 'scroll'
  },
  mainPrompt: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center'
  },
  prompt: {
    color: theme.palette.grey['200']
  },
  hidden: {
    height: 0,
    opacity: 0,
    overflow: 'hidden'
  },
  body: {
    display: 'flex',
    flexGrow: 1
  }
});

class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      selected: null
    }
  }

  componentDidMount() {
    this.props.loadApps();
    this.props.bindHistory(this.props.history);
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

  prompt() {
    return <Typography className={this.props.classes.prompt} variant="display2">
      Masquerade
    </Typography>;
  }

  features() {
    return <Fragment>
      <Hidden xsDown>
        <FeatureGroup />
      </Hidden>
      <Hidden smUp>
        <FeaturePanels />
      </Hidden>
    </Fragment>;
  }

  render() {
    let { classes, apps, app, env, loggedIn } = this.props;
    let hasSelected = app && env;
    let contentClass = classes.content + (hasSelected ? '' : ' ' + classes.mainPrompt);

    return (
      <Fragment>
        <Login />
        <CreateApp />
        <div className={loggedIn ? classes.root : classes.hidden}>
          <Header />
          <div className={classes.body}>
            <PathMenu
              menuToggle={() => {}}
              open={true} />
            <main className={contentClass}>
              {hasSelected ? this.features() : this.prompt()}
            </main>
          </div>
          <CreateMenu />
        </div>
      </Fragment>
    );
  }
}

let StyledApp = withRouter(connector(withStyles(styles)(App)));

function Run() {
  return (
    <ErrorPrompt>
      <Provider store={store}>
        <BrowserRouter>
          <MuiThemeProvider theme={theme}>
            <Updater />
            <StyledApp />
          </MuiThemeProvider>
        </BrowserRouter>
      </Provider>
    </ErrorPrompt>
  );
}

render(<Run />, document.querySelector('#app'));