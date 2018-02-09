import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import ExpansionPanel, {
  ExpansionPanelDetails,
  ExpansionPanelSummary,
} from 'material-ui/ExpansionPanel';
import List, {ListSubheader} from 'material-ui/List';
import Switch from 'material-ui/Switch';
import Typography from 'material-ui/Typography';
import ExpandMoreIcon from 'material-ui-icons/ExpandMore';

import Feature from './Feature.jsx';

let styles = theme => ({
  heading: {
    fontSize: theme.typography.pxToRem(15),
    flexBasis: '33.33%',
    flexShrink: 0,
  },
  secondaryHeading: {
    fontSize: theme.typography.pxToRem(15),
    color: theme.palette.text.secondary,
  },
});

let styled = withStyles(styles);

export default styled(function FeatureGroup(props) {
  let {classes, expanded, onChange, features, updater, app, env} = props;
  return (
    <ExpansionPanel expanded={expanded} onChange={onChange}>
      <ExpansionPanelSummary expandIcon={<ExpandMoreIcon />}>
        <Typography className={classes.heading}>{app}</Typography>
        <Typography className={classes.secondaryHeading}>{env}</Typography>
      </ExpansionPanelSummary>
      <ExpansionPanelDetails>
        <List>
          {features.map((f, i) => <Feature key={f.key} feature={f} onToggle={updater(f.key)} />)}
        </List>
      </ExpansionPanelDetails>
    </ExpansionPanel>
  );
})