import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import { TableCell, TableRow } from 'material-ui/Table';
import Switch from 'material-ui/Switch';
import IconButton from 'material-ui/IconButton';
import DeleteIcon from 'material-ui-icons/Delete';

import { connector } from './store';

const lockTime = 86400000;
// const lockTime = 60000;

class FeatureRow extends React.Component {
  shouldComponentUpdate(nextProps, nextState) {
    return !nextProps.f.enabled ||
      nextProps.f.updated_at !== this.props.f.updated_at ||
      nextProps.f.enabled !== this.props.f.enabled;
  }

  render() {
    const { classes, f, deleteFlag, updateFlag } = this.props;

    let updated = new Date();
    updated.setTime(f.updated * 1000);

    let canDelete = !f.enabled && (Date.now() - (f.updated * 1000)) > lockTime;
    let timeUntilDelete = (lockTime - (Date.now() - (f.updated * 1000))) / 1000 / 60;
    let hoursUntilDelete = Math.floor(timeUntilDelete / 60);
    let minutesUntilDelete = Math.ceil(timeUntilDelete % 60);

    return (
      <TableRow key={f.key}>
        <TableCell>{f.key}</TableCell>
        <TableCell>{f.value.toString()}</TableCell>
        <TableCell>
          <Switch
            onChange={e => updateFlag(f.key, e.target.checked)}
            checked={f.enabled}
          />
        </TableCell>
        <TableCell>{f.updated ? updated.toLocaleString() : '--'}</TableCell>
        <TableCell>
          {f.updated && canDelete && 
            <IconButton className={classes.button} aria-label="">
              <DeleteIcon onClick={() => deleteFlag(f.key)} />
            </IconButton>}
          {f.updated && !f.enabled && !canDelete && `${hoursUntilDelete}H ${minutesUntilDelete}M`}
        </TableCell>
      </TableRow>
    );
  }
}

FeatureRow.propTypes = {
  classes: PropTypes.object.isRequired,
  f: PropTypes.object.isRequired,
  deleteFlag: PropTypes.func.isRequired,
  updateFlag: PropTypes.func.isRequired
};

export default connector(withStyles()(FeatureRow));