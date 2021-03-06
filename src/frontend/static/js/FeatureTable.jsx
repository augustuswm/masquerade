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
  }
});

class FeatureTable extends React.Component {
  constructor(props) {
    super(props);
    this.filter = this.filter.bind(this);
  }

  filter(flag) {
    return flag.key.search(this.props.filterText) !== -1;
  }

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
          {flags.filter(this.filter).map(f => <FeatureRow key={f.key} f={f} />)}
        </TableBody>
      </Table>
    );
  }
}

FeatureTable.propTypes = {
  classes: PropTypes.object.isRequired,
};

export default connector(withStyles(styles)(FeatureTable));
