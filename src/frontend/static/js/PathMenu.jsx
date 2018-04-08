import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import Divider from 'material-ui/Divider';
import Drawer from 'material-ui/Drawer';
import Hidden from 'material-ui/Hidden';
import Paper from 'material-ui/Paper';
import TextField from 'material-ui/TextField';
import ListSubheader from "material-ui/List/ListSubheader";
import { ListItem, ListItemText } from "material-ui/List";

import { connector } from './store';
import PathList from './PathList.jsx';

const Fragment = React.Fragment;

const drawerWidth = 240;

const styles = theme => ({
  drawerPaper: {
    width: drawerWidth,
    [theme.breakpoints.up('md')]: {
      position: 'relative'
    }
  },
  scroll: {
    overflowY: 'scroll'
  }
});

class PathMenu extends React.Component {
  state = {
    mobileOpen: false,
  };

  handleDrawerToggle = () => {
    this.setState({ mobileOpen: !this.state.mobileOpen });
  };

  render() {
    let { classes, toggleMenu, logout } = this.props;

    return (
      <Fragment>
        <Hidden mdUp className={classes.scroll}>
          <Drawer
            variant="temporary"
            anchor={"right"}
            open={this.props.pathMenuOpen}
            onClose={() => toggleMenu(false)}
            classes={{
              paper: classes.drawerPaper,
            }}
            ModalProps={{
              keepMounted: true, // Better open performance on mobile.
            }}
          >
            <ListSubheader disableSticky={true} color="primary">
              Applications
            </ListSubheader>
            <Divider />
            <PathList
              paths={this.props.paths}
              selectPath={this.props.selectPath}
              selected={this.props.selected} />
            <Divider />
            <ListSubheader disableSticky={true} color="primary">
              Account
            </ListSubheader>
            <Divider />
            <div>
              <ListItem button onClick={() => { toggleMenu(false); logout() }}>
                <ListItemText primary="Logout" />
              </ListItem>
            </div>
          </Drawer>
        </Hidden>
        <Hidden smDown implementation="css" className={classes.scroll}>
          <Drawer
            variant="permanent"
            open
            classes={{
              paper: classes.drawerPaper
            }}
          >
            <PathList
              paths={this.props.paths}
              selectPath={this.props.selectPath}
              selected={this.props.selected} />
          </Drawer>
        </Hidden>
      </Fragment>
    );
  }
}

PathMenu.propTypes = {
  classes: PropTypes.object.isRequired,
  menuToggle: PropTypes.func.isRequired,
  open: PropTypes.bool.isRequired,
  apps: PropTypes.array.isRequired
};

export default connector(withStyles(styles, { withTheme: true })(PathMenu));