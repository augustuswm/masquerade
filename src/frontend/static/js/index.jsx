import React from 'react';
import { render } from 'react-dom';
import { Provider } from 'react-redux';
import { withStyles } from 'material-ui/styles';
import CssBaseline from 'material-ui/CssBaseline';
import { createMuiTheme, MuiThemeProvider } from 'material-ui/styles';
import AppBar from 'material-ui/AppBar';
import Toolbar from 'material-ui/Toolbar';
import Typography from 'material-ui/Typography';
import Button from 'material-ui/Button';
import IconButton from 'material-ui/IconButton';
import MenuIcon from 'material-ui-icons/Menu';
import Hidden from 'material-ui/Hidden';

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

import { connector, store } from './store';
import ErrorPrompt from './ErrorPrompt.jsx';
import FeatureGroup from './FeatureGroup.jsx';
import PathMenu from './PathMenu.jsx';
import Login from './Login.jsx';
import Updater from './Updater.jsx';
import CreateMenu from './CreateMenu.jsx';

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
  hidden: {
    height: 0,
    opacity: 0,
    overflow: 'hidden'
  },
  body: {
    display: 'flex'
  },
  top: {
    zIndex: 2000
  },
  title: {
    flex: 1
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
    let { classes, app, env, apps, flags, toggleMenu } = this.props;
    console.log(toggleMenu)

    return (
      <Fragment>
        <Login />
        <div className={apps.length > 0 ? classes.root : classes.hidden}>
          <AppBar position="static" className={classes.top}>
            <Toolbar>
              <Typography variant="title" color="inherit" className={classes.title}>
                {app} : {env}
              </Typography>
              <Button color="inherit">Account</Button>
              <Hidden mdUp>
                <IconButton color="inherit" aria-label="Apps" onClick={() => toggleMenu(true)}>
                  <MenuIcon />
                </IconButton>
              </Hidden>
            </Toolbar>
          </AppBar>
          <div className={classes.body}>
            <PathMenu
              menuToggle={() => {}}
              open={true} />
            <main className={classes.content}>
              <FeatureGroup />
            </main>
          </div>
          <CreateMenu />
        </div>
      </Fragment>
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
        <MuiThemeProvider theme={theme}>
          <Updater />
          <StyledApp />
        </MuiThemeProvider>
      </Provider>
    </ErrorPrompt>
  );
}

render(<Run />, document.querySelector('#app'));