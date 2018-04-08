import React from 'react';
import { withStyles } from 'material-ui/styles';
import SpeedDial from '@material-ui/lab/SpeedDial';
import SpeedDialIcon from '@material-ui/lab/SpeedDialIcon';
import SpeedDialAction from '@material-ui/lab/SpeedDialAction';
import LibraryAddIcon from 'material-ui-icons/LibraryAdd';

import { connector, store } from './store';

const styles = theme => ({
  speedDial: {
    position: 'absolute',
    bottom: theme.spacing.unit * 2,
    right: theme.spacing.unit * 3
  }
});

let isTouch;
if (typeof document !== 'undefined') {
  isTouch = 'ontouchstart' in document.documentElement;
}


class CreateMenu extends React.Component {
  constructor(props) {
    super(props);
    
    this.state = {
      open: false
    };
    
    this.toggle = this.toggle.bind(this);
    this.open = this.open.bind(this);
    this.close = this.close.bind(this);
  }

  toggle() {
    this.setState({ open: !this.state.open });
  }

  open() {
    this.setState({ open: true });
  }

  close() {
    this.setState({ open: false });
  }

  render() {
    let { classes, toggleAppCreate } = this.props;

    return (
      <SpeedDial
        ariaLabel="Quick Actions"
        className={classes.speedDial}
        icon={<SpeedDialIcon />}
        open={this.state.open}
        onClick={this.toggle}
        onBlur={this.close}
        onClose={this.close}
        onFocus={isTouch ? undefined : this.open}
        onMouseEnter={isTouch ? undefined : this.open}
        onMouseLeave={this.close}>
        <SpeedDialAction
          key="Add App"
          icon={<LibraryAddIcon />}
          tooltipTitle={"Add App"}
          onClick={() => toggleAppCreate(true)}
        />
      </SpeedDial>
    );
  }
}


export default connector(withStyles(styles)(CreateMenu));