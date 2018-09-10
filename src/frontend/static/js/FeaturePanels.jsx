import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import ExpansionPanel, {
  ExpansionPanelDetails,
  ExpansionPanelSummary,
} from 'material-ui/ExpansionPanel';
import Typography from 'material-ui/Typography';
import ExpandMoreIcon from 'material-ui-icons/ExpandMore';
import Grid from 'material-ui/Grid';
import Paper from 'material-ui/Paper';
import Button from 'material-ui/Button';

import FeatureCreator from './FeatureCreator.jsx';
import { connector } from './store';

const styles = theme => ({
  root: {
    flexGrow: 1
  },
  heading: {
    fontSize: theme.typography.pxToRem(15),
    flexShrink: 0
  },
  summary: {
    overflow: 'hidden'
  },
  settings: {
    flexGrow: 1
  },
  deleteItem: {
    flexGrow: 1
  },
  toggleItem: {
    flexGrow: 1
  },
  toggles: {
    marginTop: theme.spacing.unit,
    textAlign: 'center'
  }
});

// const lockTime = 86400000;
const lockTime = 60 * 1000;

class FeaturePanels extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      expanded: ''
    };

    this.expand = this.expand.bind(this);
    this.filter = this.filter.bind(this);
  }

  expand(panel) {
    this.setState({
      expanded: panel
    })
  }

  render() {
    let { expanded } = this.state;
    let { classes, app, env, flags, filterText, updateFlag } = this.props;

    return (
      <div className={classes.root}>
        <FeatureCreator />
        {flags.map(flag => {
          let updated = new Date();
          updated.setTime(flag.updated * 1000);

          let canDelete = !flag.enabled && (Date.now() - (flag.updated * 1000)) > lockTime;
          let timeUntilDelete = (lockTime - (Date.now() - (flag.updated * 1000))) / 1000 / 60;
          let hoursUntilDelete = Math.floor(timeUntilDelete / 60);
          let minutesUntilDelete = Math.ceil(timeUntilDelete % 60);

          return (
            <ExpansionPanel key={flag.key} expanded={expanded === flag.key} onChange={() => expanded === flag.key ? this.expand('') : this.expand(flag.key)}>
              <ExpansionPanelSummary className={classes.summary} expandIcon={<ExpandMoreIcon />}>
                <Typography className={classes.heading}>{flag.key}</Typography>
              </ExpansionPanelSummary>
              <ExpansionPanelDetails>
                <Grid className={classes.settings} container spacing={8}>
                  <Grid item xs={6}>
                    <Typography variant="body2">Value</Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="caption">{flag.value.toString()}</Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="body2">Enabled</Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="caption">{flag.enabled.toString()}</Typography>
                  </Grid>
                  <Grid className={classes.toggles} item xs={6}>
                    <Button
                      className={classes.deleteItem}
                      color="secondary"
                      disabled={!canDelete}
                      style={{visibility: ((flag.updated && !flag.enabled) ? 'visible' : 'hidden')}}
                      onClick={() => deleteFlag(flag.key)}>
                      {canDelete ? "Delete" : `${hoursUntilDelete}H ${minutesUntilDelete}M`}
                    </Button>
                  </Grid>
                  <Grid className={classes.toggles} item xs={6}>
                    <Button
                      className={classes.toggleItem}
                      color="secondary"
                      variant="raised"
                      onClick={() => updateFlag(flag.key, !flag.enabled)}>
                      {flag.enabled ? "Disable" : "Enable"}
                    </Button>
                  </Grid>
                </Grid>
              </ExpansionPanelDetails>
            </ExpansionPanel>
          );
        })}
      </div>
    );
  }
}

FeaturePanels.propTypes = {
  classes: PropTypes.object.isRequired,
  flags: PropTypes.array.isRequired
};

export default connector(withStyles(styles)(FeaturePanels));