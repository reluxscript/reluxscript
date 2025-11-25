using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

// ============================================================
// HOOK CLASS - Generated from useToggle
// ============================================================
[Hook]
public partial class UseToggleHook : MinimactComponent
{
    // Configuration (from hook arguments)
    private dynamic initial => GetState<dynamic>("_config.initial");

    // Hook state
    [State]
    private dynamic on = initial;

    // State setters
    private void setOn(dynamic value)
    {
        SetState(nameof(on), value);
    }

    // Hook methods
    private void toggle()
    {
        return setOn(!on);
    }

    // Hook UI rendering
    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("button", "1", new Dictionary<string, string> { ["class"] = "toggle-button", ["onclick"] = "toggle" }, new VNode[]
        {
            new VText($"{((new MObject(on)) ? "ON" : "OFF")}", "1.1")
        });
    }

}


[Component]
public partial class TestImportedHook : MinimactComponent
{
    [State]
    private int count = 0;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", "1", new Dictionary<string, string> { ["class"] = "app" }, new VNode[]
        {
            new VElement("h1", "1.1", new Dictionary<string, string>(), "Imported Hook Test"),
            new VElement("div", "1.2", new Dictionary<string, string> { ["class"] = "counter-section" }, new VNode[]
            {
                new VElement("h2", "1.2.1", new Dictionary<string, string>(), "Regular State (for comparison)"),
                new VElement("button", "1.2.2", new Dictionary<string, string> { ["onclick"] = "Handle0" }, $"Count:{(count)}")
            }),
            new VElement("div", "1.3", new Dictionary<string, string> { ["class"] = "toggle-section" }, new VNode[]
            {
                new VElement("h2", "1.3.1", new Dictionary<string, string>(), "Toggle 1 (starts OFF)"),
                new VElement("p", "1.3.2", new Dictionary<string, string>(), $"Status:{((new MObject(GetState<dynamic>("toggle1.on"))) ? "Active" : "Inactive")}"),
                new VElement("button", "1.3.3", new Dictionary<string, string> { ["onclick"] = "toggle1" }, "External Toggle 1"),
                new VComponentWrapper
      {
        ComponentName = "toggle1",
        ComponentType = "UseToggleHook",
        HexPath = "1.3.4",
        InitialState = new Dictionary<string, object> { ["_config.param0"] = false }
      }
            }),
            new VElement("div", "1.4", new Dictionary<string, string> { ["class"] = "toggle-section" }, new VNode[]
            {
                new VElement("h2", "1.4.1", new Dictionary<string, string>(), "Toggle 2 (starts ON)"),
                new VElement("p", "1.4.2", new Dictionary<string, string>(), $"Status:{((new MObject(GetState<dynamic>("toggle2.on"))) ? "Active" : "Inactive")}"),
                new VElement("button", "1.4.3", new Dictionary<string, string> { ["onclick"] = "toggle2" }, "External Toggle 2"),
                new VComponentWrapper
      {
        ComponentName = "toggle2",
        ComponentType = "UseToggleHook",
        HexPath = "1.4.4",
        InitialState = new Dictionary<string, object> { ["_config.param0"] = true }
      }
            }),
            new VElement("div", "1.5", new Dictionary<string, string> { ["class"] = "combined-state" }, new VNode[]
            {
                new VElement("p", "1.5.1", new Dictionary<string, string>(), $"Both toggles on:{(((GetState<dynamic>("toggle1.on")) != null ? (GetState<dynamic>("toggle2.on")) : new VNull("")) ? "YES" : "NO")}"),
                new VElement("p", "1.5.2", new Dictionary<string, string>(), $"At least one on:{(((GetState<dynamic>("toggle1.on")) ?? (GetState<dynamic>("toggle2.on"))) ? "YES" : "NO")}")
            })
        });
    }

    public void Handle0()
    {
        SetState(nameof(count), count + 1);
    }
}
