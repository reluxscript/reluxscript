using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Component]
public partial class UserProfile : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        var isLoading = state.isLoading;

        return MinimactHelpers.createElement("div", new { className = "user-profile" }, new VElement("h3", "1.1", new Dictionary<string, string>(), "User Profile"), new VElement("button", "1.2", new Dictionary<string, string> { ["id"] = "refresh-profile-btn", ["type"] = "button", ["onclick"] = "handleRefresh" }, "Refresh Profile"), (new MObject(isLoading)) ? new VElement("span", "1.3.1", new Dictionary<string, string> { ["class"] = "loading-indicator" }, "Loading...") : new VNull("1.3"));
    }

    public void handleRefresh()
    {
        setState("isLoading", true);
        await Task.Delay(2000);
        setState("isLoading", false);
    }
}

[Component]
public partial class ShoppingCart : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        var isLoading = state.isLoading;

        return MinimactHelpers.createElement("div", new { className = "shopping-cart" }, new VElement("h3", "1.1", new Dictionary<string, string>(), "Shopping Cart"), new VElement("button", "1.2", new Dictionary<string, string> { ["id"] = "refresh-cart-btn", ["type"] = "button", ["onclick"] = "handleRefresh" }, "Refresh Cart"), (new MObject(isLoading)) ? new VElement("span", "1.3.1", new Dictionary<string, string> { ["class"] = "loading-indicator" }, "Loading...") : new VNull("1.3"));
    }

    public void handleRefresh()
    {
        setState("isLoading", true);
        await Task.Delay(1500);
        setState("isLoading", false);
    }
}

[Component]
public partial class ContactForm : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        var isLoading = state.isLoading;

        return MinimactHelpers.createElement("div", new { className = "contact-form" }, new VElement("h3", "1.1", new Dictionary<string, string>(), "Contact Form"), new VElement("button", "1.2", new Dictionary<string, string> { ["id"] = "submit-form-btn", ["type"] = "button", ["onclick"] = "handleSubmit" }, "Submit Form"), (new MObject(isLoading)) ? new VElement("span", "1.3.1", new Dictionary<string, string> { ["class"] = "loading-indicator" }, "Loading...") : new VNull("1.3"));
    }

    public void handleSubmit()
    {
        setState("isLoading", true);
        await Task.Delay(1000);
        setState("isLoading", false);
    }
}

[Component]
public partial class Dashboard : MinimactComponent
{
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        var userLoading = state["UserProfile.isLoading"];
        var cartLoading = state["ShoppingCart.isLoading"];
        var formLoading = state["ContactForm.isLoading"];
        var anyLoading = ((userLoading) ?? (cartLoading)) ?? (formLoading);

        return MinimactHelpers.createElement("div", new { id = "dashboard-root" }, new VElement("h1", "1.1", new Dictionary<string, string>(), "Dashboard"), (new MObject(anyLoading)) ? new VElement("div", "1.2.1", new Dictionary<string, string> { ["id"] = "loading-overlay", ["class"] = "loading-overlay" }, new VNode[]
            {
                new VElement("div", "1.2.1.1", new Dictionary<string, string> { ["class"] = "spinner" }),
                new VElement("p", "1.2.1.2", new Dictionary<string, string>(), "Loading...")
            }) : new VNull("1.2"), new VComponentWrapper
{
    ComponentName = "UserProfile",
    ComponentType = "UserProfile",
    HexPath = "1.3",
    InitialState = new Dictionary<string, object> { ["isLoading"] = false }
}, new VComponentWrapper
{
    ComponentName = "ShoppingCart",
    ComponentType = "ShoppingCart",
    HexPath = "1.4",
    InitialState = new Dictionary<string, object> { ["isLoading"] = false }
}, new VComponentWrapper
{
    ComponentName = "ContactForm",
    ComponentType = "ContactForm",
    HexPath = "1.5",
    InitialState = new Dictionary<string, object> { ["isLoading"] = false }
}, new VElement("div", "1.6", new Dictionary<string, string> { ["id"] = "status", ["class"] = "status" }, new VNode[]
            {
                new VElement("p", "1.6.1", new Dictionary<string, string>(), new VNode[]
                {
                    new VText("User Loading:", "1.6.1.1"),
                    new VElement("span", "1.6.1.2", new Dictionary<string, string> { ["id"] = "user-loading" }, new VNode[]
                    {
                        new VText($"{((new MObject(userLoading)) ? "Yes" : "No")}", "1.6.1.2.1")
                    })
                }),
                new VElement("p", "1.6.2", new Dictionary<string, string>(), new VNode[]
                {
                    new VText("Cart Loading:", "1.6.2.1"),
                    new VElement("span", "1.6.2.2", new Dictionary<string, string> { ["id"] = "cart-loading" }, new VNode[]
                    {
                        new VText($"{((new MObject(cartLoading)) ? "Yes" : "No")}", "1.6.2.2.1")
                    })
                }),
                new VElement("p", "1.6.3", new Dictionary<string, string>(), new VNode[]
                {
                    new VText("Form Loading:", "1.6.3.1"),
                    new VElement("span", "1.6.3.2", new Dictionary<string, string> { ["id"] = "form-loading" }, new VNode[]
                    {
                        new VText($"{((new MObject(formLoading)) ? "Yes" : "No")}", "1.6.3.2.1")
                    })
                }),
                new VElement("p", "1.6.4", new Dictionary<string, string>(), new VNode[]
                {
                    new VText("Any Loading:", "1.6.4.1"),
                    new VElement("span", "1.6.4.2", new Dictionary<string, string> { ["id"] = "any-loading" }, new VNode[]
                    {
                        new VText($"{((new MObject(anyLoading)) ? "Yes" : "No")}", "1.6.4.2.1")
                    })
                })
            }));
    }
}
