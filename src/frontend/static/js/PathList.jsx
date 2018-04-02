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
    let apps = this.props.apps.sort((a, b) => a.path < b.path);
    let classes = this.props.classes;

    return (
      <div>
        {apps.map((app, i) => {
          return <ListItem
            key={app.path}
            button
            default={i === 0}
            onClick={() => this.props.selectApp(app.app, app.env)}
            className={this.props.selected === app.path ? classes.selected : ""}>
            <ListItemText
              primary={app.app}
              secondary={app.env}/>
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