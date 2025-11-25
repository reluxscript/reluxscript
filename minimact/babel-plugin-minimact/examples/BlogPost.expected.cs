using Minimact;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Generated.Components
{
    [Component]
    public partial class BlogPost : DefaultLayout
    {
        [State]
        private Post post = null;

        [Markdown]
        [State]
        private string markdown = @"
# Welcome to Minimact

This is **server-rendered** markdown!

- Fast
- Secure
- Reactive
  ";

        // Effect runs when [post] change
        [OnStateChanged("post")]
        private void Effect_0()
        {
            if (post != null)
            {
                Console.WriteLine($"Post \"{post.Title}\" has {post.Views} views");
            }
        }

        protected override VNode RenderContent()
        {
            return new VElement("article", new Dictionary<string, string>
            {
                ["className"] = "blog-post"
            }, new VNode[]
            {
                post != null
                    ? new Fragment(
                        new VElement("h1", $"{post.Title}"),
                        new DivRawHtml(markdown),
                        new VElement("p", $"Views: {post.Views}"),
                        new VElement("button", new Dictionary<string, string>
                        {
                            ["onClick"] = "HandleClick_0"
                        }, "Increment Views")
                    )
                    : new VElement("div", "Loading...")
            });
        }

        private void HandleClick_0()
        {
            SetState(nameof(post), new Post
            {
                Id = post.Id,
                Title = post.Title,
                Content = post.Content,
                Views = post.Views + 1
            });
        }
    }
}

// ===================================
// CODEBEHIND FILE (User creates this)
// ===================================
// BlogPost.codebehind.cs

namespace Generated.Components
{
    public partial class BlogPost
    {
        private readonly AppDbContext _db;

        public BlogPost(AppDbContext db)
        {
            _db = db;
        }

        public override async Task OnInitializedAsync()
        {
            // Load post from database
            post = await _db.Posts
                .Where(p => p.Id == RouteData.GetInt("id"))
                .FirstOrDefaultAsync();

            // Trigger re-render with loaded data
            TriggerRender();
        }
    }
}
