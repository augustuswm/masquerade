import React from 'react';
import PropTypes from 'prop-types';
import IconButton from 'material-ui/IconButton';
import CloseIcon from 'material-ui-icons/Close';
import { withStyles } from 'material-ui/styles';
import Snackbar from 'material-ui/Snackbar';

const styles = theme => ({
  close: {
    width: theme.spacing.unit * 4,
    height: theme.spacing.unit * 4,
  },
});

class ErrorPrompt extends React.Component {
  constructor(props) {
    super(props);
    this.state = {hasError: false, info: ''};
    this.removeError = this.removeError.bind(this);
    this.setError = this.setError.bind(this);
  }

  removeError() {
    this.setState({hasError: false, info: ''});
  }

  setError(err) {
    this.setState({hasError: true, info: err});
  }

  render() {
    const { classes } = this.props;

    let childrenWithProps = React.Children.map(this.props.children, child => {
      return React.cloneElement(child, {
        onError: this.setError
      });
    });

    return (
      <div>
        {childrenWithProps}
        <div>
          <Snackbar
            anchorOrigin={{
              vertical: 'bottom',
              horizontal: 'left',
            }}
            open={this.state.hasError}
            autoHideDuration={6000}
            onClose={this.removeError}
            SnackbarContentProps={{
              'aria-describedby': 'message-id',
            }}
            message={<span id="message-id">{this.state.info}</span>}
            action={[
              <IconButton
                key="close"
                aria-label="Close"
                color="inherit"
                className={classes.close}
                onClick={this.removeError}>
                <CloseIcon />
              </IconButton>
            ]}
          />
        </div>
      </div>
    );
  }
}

export default withStyles(styles)(ErrorPrompt);


