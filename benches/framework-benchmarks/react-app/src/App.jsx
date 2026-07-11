import React, { useState, useCallback } from 'react';

export default function App() {
    const [rows, setRows] = useState([]);
    const [nextId, setNextId] = useState(1);

    const createRows = useCallback((count) => {
        setRows([]);
        const newRows = [];
        let start = nextId;
        for (let i = 0; i < count; i++) {
            newRows.push({ id: start + i, label: `Row ${start + i}` });
        }
        setNextId(start + count);
        setRows(newRows);
    }, [nextId]);

    const appendRows = useCallback((count) => {
        const newRows = [];
        let start = nextId;
        for (let i = 0; i < count; i++) {
            newRows.push({ id: start + i, label: `Row ${start + i}` });
        }
        setNextId(start + count);
        setRows(prev => [...prev, ...newRows]);
    }, [nextId]);

    const updateRows = useCallback(() => {
        setRows(prev => prev.map((row, i) => {
            if (i % 10 === 0) {
                return { ...row, label: row.label + ' !!!' };
            }
            return row;
        }));
    }, []);

    const clearRows = useCallback(() => {
        setRows([]);
    }, []);

    const swapRows = useCallback(() => {
        setRows(prev => {
            if (prev.length > 998) {
                const arr = [...prev];
                const tmp = arr[1];
                arr[1] = arr[998];
                arr[998] = tmp;
                return arr;
            }
            return prev;
        });
    }, []);

    const removeRow = useCallback((id) => {
        setRows(prev => prev.filter(r => r.id !== id));
    }, []);

    return (
        <div className="container">
            <div className="jumbotron">
                <div className="row">
                    <div className="col-md-6">
                        <h1>React Benchmark</h1>
                    </div>
                    <div className="col-md-6">
                        <div className="row">
                            <div className="col-sm-6 smallpad">
                                <button type="button" className="btn btn-primary btn-block" id="run" onClick={() => createRows(1000)}>Create 1,000 rows</button>
                            </div>
                            <div className="col-sm-6 smallpad">
                                <button type="button" className="btn btn-primary btn-block" id="runlots" onClick={() => createRows(10000)}>Create 10,000 rows</button>
                            </div>
                            <div className="col-sm-6 smallpad">
                                <button type="button" className="btn btn-primary btn-block" id="add" onClick={() => appendRows(1000)}>Append 1,000 rows</button>
                            </div>
                            <div className="col-sm-6 smallpad">
                                <button type="button" className="btn btn-primary btn-block" id="update" onClick={updateRows}>Update every 10th row</button>
                            </div>
                            <div className="col-sm-6 smallpad">
                                <button type="button" className="btn btn-primary btn-block" id="clear" onClick={clearRows}>Clear</button>
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
                    {rows.map(row => (
                        <tr key={row.id}>
                            <td className="col-md-1">{row.id}</td>
                            <td className="col-md-4">
                                <a className="lbl">{row.label}</a>
                            </td>
                            <td className="col-md-1">
                                <a className="remove" onClick={() => removeRow(row.id)}>
                                    <span className="glyphicon glyphicon-remove" aria-hidden="true"></span>
                                </a>
                            </td>
                            <td className="col-md-6"></td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
