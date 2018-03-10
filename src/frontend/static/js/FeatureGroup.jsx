import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import ExpansionPanel, {
  ExpansionPanelDetails,
  ExpansionPanelSummary,
  ExpansionPanelActions,
} from 'material-ui/ExpansionPanel';
import List, {ListSubheader} from 'material-ui/List';
import Switch from 'material-ui/Switch';
import Typography from 'material-ui/Typography';
import ExpandMoreIcon from 'material-ui-icons/ExpandMore';
import Grid from 'material-ui/Grid';
import Button from 'material-ui/Button';
import Divider from 'material-ui/Divider';
import TextField from 'material-ui/TextField';
import FeatureTable from './FeatureTable.jsx';

import Feature from './Feature.jsx';

let styles = theme => ({
  expansion: {
    backgroundColor: theme.palette.grey['100']
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
    display: 'flex'
  },
  textField: {
    marginLeft: theme.spacing.unit * 2,
    marginRight: theme.spacing.unit * 2,
    width: '100%'
  }
});

let styled = withStyles(styles);

class FeatureGroup extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      newKey: ''
    };

    this.addKey = this.addKey.bind(this);
  }

  addKey(e) {
    e.preventDefault();
    this.props.adder(this.state.newKey);
    this.setState({newKey: ''});
  }

  render() {
    let {classes, expanded, onChange, features, updater, app, env, adder, remover} = this.props;
    
    return (
      <ExpansionPanel expanded={expanded} onChange={onChange}>
        <ExpansionPanelSummary expandIcon={<ExpandMoreIcon />}>
          <Typography className={classes.heading}>{app}</Typography>
          <Typography className={classes.secondaryHeading}>{env}</Typography>
        </ExpansionPanelSummary>
        <ExpansionPanelDetails>
          <FeatureTable data={features} onToggle={updater} onDelete={remover} />
        </ExpansionPanelDetails>
        <Divider />
        <ExpansionPanelActions className={classes.expansion}>
          <form className={classes.newFeatureForm} onSubmit={this.addKey}>
            <TextField
              id="key"
              label="Add New Flag"
              placeholder="Key"
              className={classes.textField}
              margin="normal"
              value={this.state.newKey}
              onChange={e => this.setState({newKey: e.target.value})}
              autoComplete="off"
            />
          </form>
        </ExpansionPanelActions>
      </ExpansionPanel>
    );
  }
}

export default styled(FeatureGroup);