import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import Paper from 'material-ui/Paper';
import TextField from 'material-ui/TextField';

import { connector } from './store';

const Fragment = React.Fragment;

const styles = theme => ({
  login: {
    position: 'absolute',
    width: '100%',
    height: '0vh',
    backgroundColor: theme.palette.primary.main,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'height',
    transitionDuration: '0.5s',
    transitionDelay: '0.5s',
    overflow: 'hidden'
  },
  loginFull: {
    height: '100vh'
  },
  loginPanel: {
    width: '100%',
    padding: theme.spacing.unit,
    display: 'flex',
    justifyContent: 'center',
    transition: 'width 0.5s linear 0s, opacity 0.25s linear 0.25s',
    opacity: 1
  },
  loginPanelHidden: {
    width: 0,
    paddingLeft: 0,
    paddingRight: 0,
    overflow: 'hidden',
    opacity: 0
  },
  loginField: {
    marginTop: 0,
    marginLeft: theme.spacing.unit,
    marginBottom: 0,
    marginRight: theme.spacing.unit,
    opacity: 1,
    transition: 'opacity',
    transitionDuration: '0.25s'
  },
  loginFieldHidden: {
    opacity: 0
  }
});

class Login extends React.Component {
  constructor(props) {
    super(props);

    this.updateKey = this.updateKey.bind(this);
    this.updateSecret = this.updateSecret.bind(this);
  }

  updateKey(e) {
    this.props.updateKey(e.target.value);
  }

  updateSecret(e) {
    this.props.updateSecret(e.target.value);
  }

  render() {
    let { classes, apiKey, apiSecret, apps } = this.props;
    let hasApps = apps.length > 0;
    let portalClasses = classes.login + (!hasApps ? ' ' + classes.loginFull : '');
    let pannelClasses = classes.loginPanel + (hasApps ? ' ' + classes.loginPanelHidden : '');
    let fieldClasses = classes.loginField + (hasApps ? ' ' + classes.loginFieldHidden : '');

    return (
      <div className={portalClasses}>
        <Paper className={pannelClasses}>
          <TextField
            className={fieldClasses}
            label="Key"
            value={apiKey}
            onChange={this.updateKey} />
          <TextField
            className={fieldClasses}
            label="Secret"
            value={apiSecret}
            onChange={this.updateSecret} />
        </Paper>
      </div>
    );
  }
}

Login.propTypes = {
  classes: PropTypes.object.isRequired,
};

export default connector(withStyles(styles)(Login));