import React from "react";
import { useState, useEffect } from "react";
import axios from "axios";
import "./styles.css";

export function fetchData(url) {
    return axios.get(url);
}

export class DataService {
    constructor() {
        this.cache = {};
    }

    async load(key) {
        return this.cache[key];
    }
}

export const API_URL = "https://api.example.com";

export default function App() {
    const [data, setData] = useState(null);

    useEffect(() => {
        fetchData(API_URL).then(setData);
    }, []);

    return <div>{data}</div>;
}
