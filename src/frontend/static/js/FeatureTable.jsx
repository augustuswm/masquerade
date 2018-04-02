import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import Table, { TableBody, TableCell, TableHead, TableRow } from 'material-ui/Table';

import FeatureRow from './FeatureRow.jsx';
import { connector } from './store';

const styles = theme => ({
  root: {
    width: '100%',
    marginTop: theme.spacing.unit * 3,
    overflowX: 'auto',
  },
});

// const lockTime = 86400000;
// const lockTime = 60000;

class FeatureTable extends React.Component {
  render() {
    const { classes, flags } = this.props;

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
          {flags.map(f => <FeatureRow key={f.key} f={f} />)}
        </TableBody>
      </Table>
    );
  }
}

FeatureTable.propTypes = {
  classes: PropTypes.object.isRequired,
};

export default connector(withStyles(styles)(FeatureTable));
