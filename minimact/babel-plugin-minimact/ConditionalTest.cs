using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Component]
public partial class ConditionalTest : MinimactComponent
{
    [State]
    private bool myState1 = false;

    [State]
    private bool myState2 = false;

    [State]
    private string myState3 = "Hello World";

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return MinimactHelpers.createElement("div", null, new VElement("h1", "1.1", new Dictionary<string, string>(), "Conditional Element Test"), (new MObject(myState1)) ? new VElement("div", "1.2.1", new Dictionary<string, string>(), "myState1 is true") : new VNull("1.2"), ((myState1) && (!myState2)) ? new VElement("div", "1.3.1", new Dictionary<string, string> { ["class"] = "nested-content" }, new VNode[]
            {
                new VElement("span", "1.3.1.1", new Dictionary<string, string>(), "SomeNestedDOMElementsHere"),
                new VElement("span", "1.3.1.2", new Dictionary<string, string>(), new VNode[]
                {
                    new VText($"{(myState3)}", "1.3.1.2.1")
                })
            }) : new VNull("1.3"), (new MObject(myState1)) ? new VElement("div", "1.4.1", new Dictionary<string, string> { ["class"] = "active" }, "Active State") : new VElement("div", "1.4.2", new Dictionary<string, string> { ["class"] = "inactive" }, "Inactive State"), new VElement("button", "1.5", new Dictionary<string, string> { ["onclick"] = "Handle0" }, "Toggle myState1"));
    }

    public void Handle0()
    {
        SetState(nameof(myState1), !myState1);
    }
}
