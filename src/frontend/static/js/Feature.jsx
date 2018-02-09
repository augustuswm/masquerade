import React from 'react';
import PropTypes from 'prop-types';
import { withStyles } from 'material-ui/styles';
import { ListItem, ListItemIcon, ListItemSecondaryAction, ListItemText } from 'material-ui/List';
import Switch from 'material-ui/Switch';

export default function Feature({feature, onToggle}) {
  return (
    <ListItem>
      <ListItemText primary={feature.key} />
      <ListItemSecondaryAction>
        <Switch
          onChange={e => onToggle(e.target.checked)}
          checked={feature.enabled}
        />
      </ListItemSecondaryAction>
    </ListItem>
  );
}