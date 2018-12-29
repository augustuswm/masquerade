import React from 'react';

import { connector } from './store';

class Updater extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      stream: null
    };

    this.update = this.update.bind(this);
  }

  shouldComponentUpdate(nextProps) {
    return this.props.app !== nextProps.app ||
      this.props.env !== nextProps.env ||
      this.props.token !== nextProps.token;
  }

  componentDidMount() {
  }

  componentDidUpdate() {
    console.log('updated');

    // Close any existing connections
    if (this.state.stream && this.state.stream.close) {
      this.state.stream.close();
    }

    let { app, env, token, baseUrl } = this.props;

    if (app && env && token) {
      let stream = new EventSource(`${window.location.origin}${baseUrl}/stream/${app}/${env}/?token=${token}`);
      stream.addEventListener('data', e => this.update(e.data));

      this.setState({
        stream: stream
      });
    }
  }

  update(data) {
    this.props.loadFlags(JSON.parse(data))
  }

  render() {
    return <div></div>;
  }
}

export default connector(Updater);