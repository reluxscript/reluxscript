# Generated Schema

type User {
  id: Number
  name: String
  email: String
  active: Boolean
}

type Product {
  sku: String
  title: String
  price: Number
  inStock: Boolean
  tags: Array
}

type Order {
  orderId: String
  customerId: Number
  items: Array
  total: Number
  status: String
}

