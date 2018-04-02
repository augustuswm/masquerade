import React from 'react';

import { connector } from './store';

class Updater extends React.Component {
  constructor(props) {
    super(props);

    this.run = this.run.bind(this);
    this.schedule = this.schedule.bind(this);
  }

  componentDidMount() {
    // this.schedule();
  }

  run() {
    this.props.loadFlags(this.props.app, this.props.env);
    this.schedule();
  }

  schedule() {
    setTimeout(this.run, this.props.refresh);
  }

  render() {
    return <div></div>;
  }
}

export default connector(Updater);