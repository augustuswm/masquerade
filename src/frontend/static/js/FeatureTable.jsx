import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import Table, { TableBody, TableCell, TableHead, TableRow } from 'material-ui/Table';
import Paper from 'material-ui/Paper';
import Switch from 'material-ui/Switch';
import IconButton from 'material-ui/IconButton';
import DeleteIcon from 'material-ui-icons/Delete';
import Tooltip from 'material-ui/Tooltip';

const styles = theme => ({
  root: {
    width: '100%',
    marginTop: theme.spacing.unit * 3,
    overflowX: 'auto',
  },
});

// const lockTime = 86400000;
const lockTime = 60000;

function FeatureTable(props) {
  const { classes, onToggle, onDelete, data } = props;

  return (
      <Table className={classes.table}>
        <TableHead>
          <TableRow>
            <TableCell>Key</TableCell>
            <TableCell>Value</TableCell>
            <TableCell>Enabled</TableCell>
            <TableCell>Updated</TableCell>
            <TableCell>Delete</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {data.map(f => {
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
                    onChange={e => onToggle(f.key)(e.target.checked)}
                    checked={f.enabled}
                  />
                </TableCell>
                <TableCell>{f.updated ? updated.toLocaleString() : '--'}</TableCell>
                <TableCell>
                  {f.updated && canDelete && 
                    <IconButton className={classes.button} aria-label="">
                      <DeleteIcon onClick={() => onDelete(f.key)} />
                    </IconButton>}
                  {f.updated && !f.enabled && !canDelete && `${hoursUntilDelete}H ${minutesUntilDelete}M`}
                </TableCell>
              </TableRow>
            );
          })}
        </TableBody>
      </Table>
  );
}

FeatureTable.propTypes = {
  classes: PropTypes.object.isRequired,
};

export default withStyles(styles)(FeatureTable);
