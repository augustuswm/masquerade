import React from 'react';
import PropTypes from 'prop-types';
import { withRouter } from 'react-router-dom';

import { withStyles } from 'material-ui/styles';
import AppBar from 'material-ui/AppBar';
import Toolbar from 'material-ui/Toolbar';
import Typography from 'material-ui/Typography';
import Button from 'material-ui/Button';
import IconButton from 'material-ui/IconButton';
import MenuIcon from 'material-ui-icons/Menu';
import Hidden from 'material-ui/Hidden';
import Input from 'material-ui/Input';
import Paper from 'material-ui/Paper';

import { connector } from './store';

const styles = theme => ({
  top: {
    zIndex: 2000
  },
  title: {
    flex: 1
  },
  searchbox: {
    padding: '0 ' + theme.spacing.unit,
    background: 'rgba(255,255,255, 0.5)',
    [theme.breakpoints.down('sm')]: {
      display: 'none'
    }
  },
  header: {
    [theme.breakpoints.down('sm')]: {
      paddingRight: 0
    }
  }
});

class Header extends React.Component {
  render() {
    let { classes, logout, app, env, toggleMenu, filterText, updateFilter, history } = this.props;

    return (
      <AppBar position="static" className={classes.top}>
        <Toolbar className={classes.header}>
          <div className={classes.title}>
            <Typography variant="title" color="inherit">
              {app}
            </Typography>
            <Typography variant="subheading" color="inherit">
              {env}
            </Typography>
          </div>
          <Paper className={classes.searchbox}>
            <Input
              disableUnderline={true}
              placeholder="Filter"
              label="Filter"
              values={filterText}
              onChange={e => updateFilter(history, e.target.value)} />
          </Paper>
          <Hidden smDown>
            <Button color="inherit" onClick={logout}>Logout</Button>
          </Hidden>
          <Hidden mdUp>
            <IconButton color="inherit" aria-label="Apps" onClick={() => toggleMenu(true)}>
              <MenuIcon />
            </IconButton>
          </Hidden>
        </Toolbar>
      </AppBar>
    );
  }
}

Header.propTypes = {
  classes: PropTypes.object.isRequired,
  app: PropTypes.string.isRequired,
  env: PropTypes.string.isRequired,
  logout: PropTypes.func.isRequired,
  toggleMenu: PropTypes.func.isRequired
};

export default withRouter(connector(withStyles(styles, { withTheme: true })(Header)));