import React from 'react';
import { render } from 'react-dom';
import FeatureGroup from './FeatureGroup.jsx';
import Store from './Store.jsx';

let data = [
  {
    app: "nextavenue",
    env: "prod",
    features: [
      {"key":"f5","value":true,"version":1,"enabled":false},
      {"key":"f3","value":false,"version":2,"enabled":true},
      {"key":"f9","value":true,"version":4,"enabled":false},
      {"key":"f7","value":false,"version":2,"enabled":true}
    ]
  }
];

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
      <div>
        {
          this.props.data.map(group => {
            let key = `${group.app}::${group.env}`;
            let expanded = this.isExpanded(key);

            return <FeatureGroup
              key={key}
              app={group.app}
              env={group.env}
              expanded={expanded}
              onChange={this.setExpanded(key)}
              features={group.features}
              updater={this.props.onUpdate(group.app, group.env)}
            />;
          })
        }
      </div>
    );
  }
}

function Run() {
  return (
    <Store data={data} baseUrl="/api/v1">
      <App />
    </Store>
  );
}

render(<Run />, document.querySelector('#app'));