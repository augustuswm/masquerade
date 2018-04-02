import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import Drawer from 'material-ui/Drawer';
import Hidden from 'material-ui/Hidden';
import PathList from './PathList.jsx';

import { connector } from './store';

const Fragment = React.Fragment;

const drawerWidth = 240;

const styles = theme => ({
  drawerPaper: {
    width: drawerWidth,
    [theme.breakpoints.up('md')]: {
      position: 'relative'
    }
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
    let classes = this.props.classes;

    return (
      <Fragment>
        <Hidden mdUp>
          <Drawer
            variant="temporary"
            anchor={"right"}
            open={this.props.open}
            onClose={this.props.menuToggle}
            classes={{
              paper: classes.drawerPaper,
            }}
            ModalProps={{
              keepMounted: true, // Better open performance on mobile.
            }}
          >
            <PathList
              paths={this.props.paths}
              selectPath={this.props.selectPath}
              selected={this.props.selected} />
          </Drawer>
        </Hidden>
        <Hidden smDown implementation="css">
          <Drawer
            variant="permanent"
            open
            classes={{
              paper: classes.drawerPaper,
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

export default connector(withStyles(styles)(PathMenu));