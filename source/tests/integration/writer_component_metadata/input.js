function MyComponent(props) {
    const [count, setCount] = React.useState(0);
    const [name, setName] = React.useState("");

    React.useEffect(() => {
        console.log(count);
    }, [count]);

    return <div>{count}</div>;
}

function AnotherComponent(props) {
    React.useEffect(() => {
        console.log("mounted");
    }, []);

    return <span>Hello</span>;
}

function helperFunction() {
    return 42;
}
