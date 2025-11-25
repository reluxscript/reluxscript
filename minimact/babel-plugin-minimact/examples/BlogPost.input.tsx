// Example: Blog post component with EF Core, Markdown, and Template
import { useState, useEffect, useMarkdown, useTemplate } from '@minimact/core';

interface Post {
  id: number;
  title: string;
  content: string;
  views: number;
}

export function BlogPost() {
  // Will be populated from database in codebehind
  const [post, setPost] = useState<Post | null>(null);

  // Markdown content that gets parsed server-side
  const [markdown, setMarkdown] = useMarkdown(`
# Welcome to Minimact

This is **server-rendered** markdown!

- Fast
- Secure
- Reactive
  `);

  // Use the "DefaultLayout" template
  useTemplate("DefaultLayout", { title: "Blog Post" });

  // Track view count
  useEffect(() => {
    if (post) {
      console.log(`Post "${post.title}" has ${post.views} views`);
    }
  }, [post]);

  return (
    <article className="blog-post">
      {post ? (
        <>
          <h1>{post.title}</h1>
          <div markdown>{markdown}</div>
          <p>Views: {post.views}</p>
          <button onClick={() => setPost({ ...post, views: post.views + 1 })}>
            Increment Views
          </button>
        </>
      ) : (
        <div>Loading...</div>
      )}
    </article>
  );
}
