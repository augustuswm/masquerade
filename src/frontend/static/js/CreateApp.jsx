import React from 'react';
import PropTypes from 'prop-types';

import { withStyles } from 'material-ui/styles';
import Button from 'material-ui/Button';
import Dialog, {
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
} from 'material-ui/Dialog';
import TextField from 'material-ui/TextField';
import Typography from 'material-ui/Typography';

import { connector } from './store';

let styles = theme => ({
});

let validChars = /^[a-z0-9_-]$/;

let prevent = e => e.preventDefault();

class CreateApp extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      app: "",
      env: ""
    };

    this.setApp = this.setApp.bind(this);
    this.setEnv = this.setEnv.bind(this);
  }

  updateField(field, val) {

    // Shortcut the empty string case
    if (val.length === 0) {
      if (this.state.val !== val) {
        this.setState({ [field]: val });
      }

      return;
    }

    let c = val[val.length - 1];

    // Replace space with underscore
    if (c === " ") {
      val = val.substr(0, val.length - 1) + "_";
    }

    // // Validate the input
    if (val[val.length - 1].search(validChars) !== -1) {
      this.setState({ [field]: val });
    }
  }

  clearFields() {
    this.setState({
      app: "",
      env: ""
    });
  }

  setApp(e) {
    this.updateField('app', e.target.value);
  }

  setEnv(e) {
    this.updateField('env', e.target.value);
  }

  checkForExistance(app, env) {
    return this.props.apps.filter(a => {
      return a.app === app && a.env === env;
    }).length > 0;
  }

  render() {
    let { classes, appCreateModalOpen, toggleAppCreate, addApp } = this.props;
    let { app, env } = this.state;
    let exists = this.checkForExistance(app, env);
    let allowCreate = app.length > 0 && env.length > 0 && !exists;

    return (
      <Dialog
        open={appCreateModalOpen}
        onClose={() => toggleAppCreate(false)}
        aria-labelledby="form-dialog-title">
        <DialogTitle id="form-dialog-title">Add Application</DialogTitle>
        <DialogContent>
          <form
            onSubmit={allowCreate ? () => addApp(app, env) : prevent}>
            <TextField
              autoFocus
              margin="dense"
              id="name"
              label={exists ? "App / Env pair already exists" : "Application"}
              fullWidth
              value={app}
              onChange={this.setApp}
              error={exists} />
            <TextField
              margin="dense"
              id="name"
              label={exists ? "App / Env pair already exists" : "Environment"}
              fullWidth
              value={env}
              onChange={this.setEnv}
              error={exists} />
          </form>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => { this.clearFields(); toggleAppCreate(false); }} color="primary">
            Cancel
          </Button>
          <Button onClick={allowCreate ? () => addApp(app, env) : prevent} color="primary">
            Add
          </Button>
        </DialogActions>
      </Dialog>
    );
  }
}

export default connector(withStyles(styles)(CreateApp));