import React from 'react';
import PropTypes from 'prop-types';

import { withStyles } from 'material-ui/styles';
import Button from 'material-ui/Button';
import Divider from 'material-ui/Divider';
import ExpandMoreIcon from 'material-ui-icons/ExpandMore';
import ExpansionPanel, {
  ExpansionPanelDetails,
  ExpansionPanelSummary,
  ExpansionPanelActions,
} from 'material-ui/ExpansionPanel';
import Grid from 'material-ui/Grid';
import List, {ListSubheader} from 'material-ui/List';
import Paper from 'material-ui/Paper';
import Switch from 'material-ui/Switch';
import TextField from 'material-ui/TextField';
import Typography from 'material-ui/Typography';
import Hidden from 'material-ui/Hidden';

import FeatureCreator from './FeatureCreator.jsx';
import FeatureTable from './FeatureTable.jsx';
import Feature from './Feature.jsx';
import { connector } from './store';

const Fragment = React.Fragment;

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
  heading: {
    fontSize: theme.typography.pxToRem(15),
    flexBasis: '33.33%',
    flexShrink: 0,
  },
  secondaryHeading: {
    fontSize: theme.typography.pxToRem(15),
    color: theme.palette.text.secondary,
  },
  list: {
    width: '100%'
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
  },
  reminder: {
    color: theme.palette.grey['400'],
    textAlign: 'center',
    margin: theme.spacing.unit * 2
  }
});

let validChars = /^[a-z0-9_]$/;

let prevent = e => e.preventDefault();

class FeatureGroup extends React.Component {
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
    let { classes, flags } = this.props;
    let exists = this.state.existingKeys.indexOf(this.state.newKey) !== -1;
    let allowCreate = this.state.newKey.length > 0 && !exists;
    let filtered = flags.filter(flag => flag.key.search(this.props.filterText) !== -1);
    let hidden = flags.length - filtered.length;
    
    return (
      <Fragment>
        <Paper>
          <FeatureCreator />
          <Divider />
          <FeatureTable flags={filtered} />
        </Paper>
        <Typography className={classes.reminder}>
          ({hidden} flag{hidden === 1 ? " is " : "s are "}hidden)
        </Typography>
      </Fragment>
    );
  }
}

export default connector(withStyles(styles)(FeatureGroup));