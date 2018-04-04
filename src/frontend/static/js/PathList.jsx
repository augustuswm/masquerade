import React from "react";
import PropTypes from "prop-types";
import { withStyles } from "material-ui/styles";
import Collapse from "material-ui/transitions/Collapse";
import Divider from 'material-ui/Divider';
import ExpandLess from "material-ui-icons/ExpandLess";
import ExpandMore from "material-ui-icons/ExpandMore";
import ListSubheader from "material-ui/List/ListSubheader";
import List, { ListItem, ListItemIcon, ListItemText } from "material-ui/List";

import { connector } from './store';

const Fragment = React.Fragment;

const styles = theme => ({
  nested: {
    paddingLeft: theme.spacing.unit
  },
  selected: {
    backgroundColor: theme.palette.grey['200'],
  }
});

class PathList extends React.Component {
  render() {
    let { className, classes, apps, app, env, selectApp, toggleMenu } = this.props;

    return (
      <div className={className}>
        {apps.map((a, i) => {
          let selected = a.app === app && a.env === env;

          return <ListItem
            key={a.path}
            button
            default={i === 0}
            onClick={() => { toggleMenu(false); selectApp(a.app, a.env); }}
            className={selected ? classes.selected : ""}>
            <ListItemText
              primary={a.app}
              secondary={a.env}/>
          </ListItem>
        })}
      </div>
    );
  }
}

PathList.propTypes = {
  classes: PropTypes.object.isRequired,
  apps: PropTypes.array.isRequired
};

export default connector(withStyles(styles)(PathList));