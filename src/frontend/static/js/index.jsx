import React from 'react';
import { render } from 'react-dom';
import Reboot from 'material-ui/Reboot';
import ErrorPrompt from './ErrorPrompt.jsx';
import FeatureGroup from './FeatureGroup.jsx';
import Store from './Store.jsx';

class App extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      expanded: null
    }
  }

  isExpanded(key) {
    return this.state.expanded === key;
  }

  setExpanded(key) {
    return () => {
      this.setState((prevState, props) => {
        return {
          expanded: prevState.expanded !== key ? key : null
        };
      })
    }
  }

  render() {
    return (
      <div style={{maxWidth: '800', margin: '0 auto'}}>
        {
          this.props.data.map(group => {
            let key = `${group.app}::${group.env}`;
            let expanded = this.isExpanded(key);
            let adder = key => {
              this.props.onAdd(group.app, group.env, key);
            };
            let remover = key => {
              this.props.onDelete(group.app, group.env, key);
            };

            return <FeatureGroup
              key={key}
              app={group.app}
              env={group.env}
              expanded={expanded}
              onChange={this.setExpanded(key)}
              features={group.features}
              updater={this.props.onUpdate(group.app, group.env)}
              adder={adder}
              remover={remover}
            />;
          })
        }
      </div>
    );
  }
}

function Run() {
  return (
    <ErrorPrompt>
      <Store baseUrl="/api/v1">
        <App />
      </Store>
    </ErrorPrompt>
  );
}

render(<Run />, document.querySelector('#app'));