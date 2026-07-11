import { Store } from '@tanstack/store'
import { useStore } from '@tanstack/react-store'
import React from 'react'

const store = new Store({
  data: [],
  selected: 0,
});

let idCounter = 1;

function buildData(count) {
  const data = new Array(count);
  for (let i = 0; i < count; i++) {
    data[i] = { id: idCounter++, label: `Row ${idCounter - 1}` };
  }
  return data;
}

export default function App() {
  const rows = useStore(store, (state) => state.data);

  const run = () => store.setState((state) => ({ ...state, data: buildData(1000) }));
  const runLots = () => store.setState((state) => ({ ...state, data: buildData(10000) }));
  const add = () => store.setState((state) => ({ ...state, data: state.data.concat(buildData(1000)) }));
  const update = () => store.setState((state) => {
    const newData = state.data.slice();
    for (let i = 0; i < newData.length; i += 10) {
      newData[i] = { ...newData[i], label: newData[i].label + ' !!!' };
    }
    return { ...state, data: newData };
  });
  const clear = () => store.setState((state) => ({ ...state, data: [] }));
  const swapRows = () => store.setState((state) => {
    const newData = state.data.slice();
    if (newData.length > 998) {
      let temp = newData[1];
      newData[1] = newData[998];
      newData[998] = temp;
    }
    return { ...state, data: newData };
  });
  const remove = (id) => store.setState((state) => {
    return { ...state, data: state.data.filter((d) => d.id !== id) };
  });

  return (
    <div className="container">
      <div className="jumbotron">
        <div className="row">
          <div className="col-md-6">
            <h1>TanStack Benchmark</h1>
          </div>
          <div className="col-md-6">
            <div className="row">
              <div className="col-sm-6 smallpad">
                <button type="button" className="btn btn-primary btn-block" id="run" onClick={run}>Create 1,000 rows</button>
              </div>
              <div className="col-sm-6 smallpad">
                <button type="button" className="btn btn-primary btn-block" id="runlots" onClick={runLots}>Create 10,000 rows</button>
              </div>
              <div className="col-sm-6 smallpad">
                <button type="button" className="btn btn-primary btn-block" id="add" onClick={add}>Append 1,000 rows</button>
              </div>
              <div className="col-sm-6 smallpad">
                <button type="button" className="btn btn-primary btn-block" id="update" onClick={update}>Update every 10th row</button>
              </div>
              <div className="col-sm-6 smallpad">
                <button type="button" className="btn btn-primary btn-block" id="clear" onClick={clear}>Clear</button>
              </div>
              <div className="col-sm-6 smallpad">
                <button type="button" className="btn btn-primary btn-block" id="swaprows" onClick={swapRows}>Swap Rows</button>
              </div>
            </div>
          </div>
        </div>
      </div>
      <table className="table table-hover table-striped test-data">
        <tbody>
          {rows.map((row) => (
            <tr key={row.id}>
              <td className="col-md-1">{row.id}</td>
              <td className="col-md-4">
                <a className="lbl">{row.label}</a>
              </td>
              <td className="col-md-1">
                <a className="remove" onClick={() => remove(row.id)}>
                  <span className="glyphicon glyphicon-remove" aria-hidden="true" />
                </a>
              </td>
              <td className="col-md-6" />
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
