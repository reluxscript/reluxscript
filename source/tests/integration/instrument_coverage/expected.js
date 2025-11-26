function add(a, b) {
    __coverage__.f[0]++;
    __coverage__.s[0]++;
    return a + b;
}

__coverage__.s[1]++;
const result = add(1, 2);
