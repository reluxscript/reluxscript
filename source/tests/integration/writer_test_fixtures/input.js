// Type-like objects that we'll generate fixtures from

const UserSchema = {
    id: 0,
    username: "",
    email: "",
    isActive: false,
};

const ProductSchema = {
    productId: 0,
    name: "",
    price: 0.0,
    categories: [],
    metadata: {},
};

const CommentSchema = {
    commentId: 0,
    userId: 0,
    text: "",
    likes: 0,
    replies: [],
};
