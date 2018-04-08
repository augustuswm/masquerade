import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import Button from 'material-ui/Button';
import TextField from 'material-ui/TextField';
import Hidden from 'material-ui/Hidden';

import { connector } from './store';

let styles = theme => ({
  addButton: {
    margin: theme.spacing.unit,
    marginRight: theme.spacing.unit * 2
  },
  expansion: {
    display: 'flex',
    backgroundColor: theme.palette.grey['100'],
    alignItems: 'flex-end'
  },
  newFeatureForm: {
    width: '100%',
    margin: 0,
    display: 'flex',
  },
  textField: {
    marginLeft: theme.spacing.unit * 2,
    marginRight: theme.spacing.unit * 2,
    width: '100%'
  }
});

let validChars = /^[a-z0-9_]$/;

let prevent = e => e.preventDefault();

class FeatureCreator extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      newKey: '',
      existingKeys: []
    };

    this.addKey = this.addKey.bind(this);
    this.inputChange = this.inputChange.bind(this);
  }

  addKey(e) {
    e.preventDefault();
    this.props.addFlag(this.state.newKey);
    this.setState({newKey: ''});
  }

  inputChange(e) {
    e.preventDefault();

    let val = e.target.value;

    // Shortcut the empty string case
    if (val.length === 0) {
      if (this.state.val !== val) {
        this.setState({newKey: val});
      }

      return;
    }

    let c = val[val.length - 1];

    // Replace space with underscore
    if (c === " " || c === "-") {
      val = val.substr(0, val.length - 1) + "_";
    }

    // // Validate the input
    if (val[val.length - 1].search(validChars) !== -1) {
      this.setState({ newKey: val });
    }
  }

  static getDerivedStateFromProps(nextProps, prevState) {
    return {
      existingKeys: nextProps.flags.map(f => f.key)
    };
  }

  render() {
    let { classes } = this.props;
    let exists = this.state.existingKeys.indexOf(this.state.newKey) !== -1;
    let allowCreate = this.state.newKey.length > 0 && !exists;

    return (
      <div className={classes.expansion}>
        <form
          className={classes.newFeatureForm}
          onSubmit={allowCreate ? this.addKey : prevent}>
          <TextField
            id="key"
            label={exists ? "Flag with this key already exists" : "Add New Flag"}
            placeholder="Key"
            className={classes.textField}
            margin="normal"
            value={this.state.newKey}
            onChange={this.inputChange}
            autoComplete="off"
            error={exists}
          />
        </form>
        <Hidden xsDown implementation="css">
          <Button
            className={classes.addButton}
            variant="raised"
            color="secondary"
            disabled={!allowCreate}
            onClick={this.addKey}>
            Add
          </Button>
        </Hidden>
      </div>
    );
  }
}

export default connector(withStyles(styles)(FeatureCreator));