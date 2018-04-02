import React from 'react';
import { render } from 'react-dom';
import axios from 'axios';

export default class Store extends React.Component {
  initialRefresh = 1000;

  constructor(props) {
    super(props);

    this.state = {
      data: props.data || [],
      refresh: this.initialRefresh
    }

    this.addFeature = this.addFeature.bind(this);
    this.deleteFeature = this.deleteFeature.bind(this);
    this.onUpdate = this.onUpdate.bind(this);
    this.writeFeature = this.writeFeature.bind(this);
  }

  addFeature(app, env, key) {
    if (this.props.baseUrl) {
      let url = `${this.props.baseUrl}/${app}/${env}/flag/`;

      let data = this.state.data;
      let appIndex = data.findIndex(entry => { return entry.app === app && entry.env === env; });
      let entry = data[appIndex];
      let flag = {
        key, app, env, value: true, version: 1, enabled: true
      };

      entry.features.push(flag);
      // entry.features.sort((a, b) => {
      //   return a.key > b.key;
      // });

      data[appIndex] = entry;
      this.setState({data: data});
      
      return axios.post(url, flag).catch(err => {
        this.props.onError(`Failed to create flag ${flag.key}`);
      });
    }

    return Promise.reject(false);
  }

  componentDidMount() {
    this.loadApps();
  }

  deleteFeature(app, env, key) {
    if (this.props.baseUrl) {
      let url = `${this.props.baseUrl}/${app}/${env}/flag/${key}/`;

      let data = this.state.data;
      let appIndex = data.findIndex(entry => { return entry.app === app && entry.env === env; });
      let entry = data[appIndex];
      let featureIndex = entry.features.findIndex(f => f.key === key);

      delete entry.features[featureIndex];
      data[appIndex] = entry;

      this.setState({data: data});
      
      return axios.delete(url).catch(err => {
        this.props.onError(`Failed to delete flag ${key}`);
      });
    }

    return Promise.reject(false);
  }

  loadApps() {
    let url = `${this.props.baseUrl}/paths/`;
    axios.get(url)
      .then(resp => {
        let apps = resp.data.map(app => {
          app.features = [];
          return app;
        });
        
        apps.sort((a, b) => {
          if (a.app !== b.app) {
            return a.app > b.app;
          } else {
            return a.env > b.env;
          }
        });

        this.setState({data: apps});
        apps.forEach(app => this.scheduleUpdate(app.app, app.env, 1000));
      })
  }

  loadData(app, env) {
    let url = `${this.props.baseUrl}/${app}/${env}/flags/`;
    axios.get(url)
      .then(resp => {
        // resp.data.sort((a, b) => a.key > b.key);

        let appIndex = this.state.data.findIndex(entry => { return entry.app === app && entry.env === env; });
        this.state.data[appIndex].features = resp.data;

        this.setState({data: this.state.data, refresh: this.initialRefresh});
        this.scheduleUpdate(app, env, this.state.refresh);
      })
      .catch(err => {
        let newRefresh = this.state.refresh * 2;

        this.setState({refresh: newRefresh})
        this.scheduleUpdate(app, env, newRefresh);

        this.props.onError("Failed to refresh");
      });
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

  scheduleUpdate(app, env, delay) {
    setTimeout(() => {
      this.loadData(app, env);
    }, delay)
  }

  writeFeature(app, env, key, flag) {
    if (this.props.baseUrl) {
      let url = `${this.props.baseUrl}/${app}/${env}/flag/${flag.key}/`;

      flag.app = app;
      flag.env = env;

      return axios.post(url, flag).catch(err => {
        this.props.onError(`Failed to update flag ${flag.key}`);
      });
    }

    return Promise.reject(false);
  }

  render() {
    let childrenWithProps = React.Children.map(this.props.children, child => {
      return React.cloneElement(child, {
        data: this.state.data,
        onUpdate: this.onUpdate,
        onAdd: this.addFeature,
        onDelete: this.deleteFeature
      });
    });

    return (
      <div>
        {childrenWithProps}
      </div>
    );
  }
}