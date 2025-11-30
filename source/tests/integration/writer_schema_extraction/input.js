// Data shapes that define API contracts

const User = {
    id: 123,
    name: "John Doe",
    email: "john@example.com",
    active: true,
};

const Product = {
    sku: "ABC-123",
    title: "Widget",
    price: 29.99,
    inStock: true,
    tags: ["electronics", "gadgets"],
};

const Order = {
    orderId: "ORD-001",
    customerId: 456,
    items: [],
    total: 99.99,
    status: "pending",
};

// Not a schema (function call)
const result = someFunction();
