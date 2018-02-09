import React from 'react';
import { render } from 'react-dom';
import axios from 'axios';

export default class Store extends React.Component {
  constructor(props) {
    super(props);
    this.state = {
      data: props.data
    }

    this.onUpdate = this.onUpdate.bind(this);
    this.writeFeature = this.writeFeature.bind(this);
  }

  loadData(app, env) {
    let url = `${this.props.baseUrl}/${app}/${env}/flags/`;
    axios.get(url)
      .then(resp => {
        resp.data.sort((a, b) => a.key > b.key);
        this.setState({data: [{app, env, features: resp.data}]});
      });
  }

  componentDidMount() {
    setInterval(() => {
      this.loadData("", "");
    }, 500)
  }

  onUpdate(app, env) {
    let d = this.state.data.slice();

    let groups = d.filter(g => g.app === app && g.env === env);

    if (groups.length > 0) {
      let group = groups[0];

      return key => {
        let fs = group.features.filter(f => f.key === key);

        if (fs.length > 0) {
          let f = fs[0];

          return enabled => {
            f.enabled = enabled;

            this.setState({data: d});
            this.writeFeature(app, env, key, f).catch(e => {
              f.enabled = !enabled;
              this.setState({data: d});
            });
          }
        } else {
          return () => undefined;
        }
      }
    } else {
      return () => undefined;
    }
  }

  writeFeature(app, env, key, flag) {
    if (this.props.baseUrl) {
      let url = `${this.props.baseUrl}/${app}/${env}/flag/${flag.key}/`;

      flag.app = app;
      flag.env = env;

      return axios.post(url, flag);
    }

    return Promise.reject(false);
  }

  render() {
    let childrenWithProps = React.Children.map(this.props.children, child => {
      return React.cloneElement(child, {
        data: this.state.data,
        onUpdate: this.onUpdate
      });
    });

    return (
      <div>
        {childrenWithProps}
      </div>
    );
  }
}