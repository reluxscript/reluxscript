using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Component]
public partial class Index : MinimactComponent
{
    [State]
    private dynamic activeExample = new VNull("");

    [State]
    private int viewCount = 0;

    [Ref]
    private object modalRef = new VNull("");

    [Ref]
    private object timerRef = 0;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return MinimactHelpers.createElement("div", new { style = "padding: 20px; font-family: system-ui, sans-serif; max-width: 1200px; margin: 0 auto" }, new VElement("h1", "1.1", new Dictionary<string, string> { ["style"] = "margin-bottom: 10px" }, "Minimact Hook Examples"), new VElement("p", "1.2", new Dictionary<string, string> { ["style"] = "color: #666; margin-bottom: 10px" }, "This project includes examples for 3 hooks.\n        Select an example below to see the code in action."), new VElement("p", "1.2.1", new Dictionary<string, string> { ["style"] = "color: #999; font-size: 14px; margin-bottom: 30px" }, $"Page views:{(viewCount)}| Timer ref:{(timerRef)}"), new VElement("div", "1.3", new Dictionary<string, string> { ["style"] = "display: grid; gap: 30px" }, new VNode[]
            {
                new VElement("div", "1.3.1", new Dictionary<string, string>(), new VNode[]
                {
                    new VElement("h2", "1.3.1.1", new Dictionary<string, string> { ["style"] = "font-size: 20px; margin-bottom: 16px; color: #333" }, "Core Hooks"),
                    new VElement("div", "1.3.1.2", new Dictionary<string, string> { ["style"] = "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 12px" }, new VNode[]
                    {
                        new VElement("button", "1.3.1.2.1", new Dictionary<string, string> { ["style"] = "padding: 12px 16px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; text-align: left; transition: all 0.2s", ["onclick"] = "Handle0", ["onmouseenter"] = "@client:Handle1", ["onmouseleave"] = "@client:Handle2" }, new VNode[]
                        {
                            new VElement("div", "1.3.1.2.1.1", new Dictionary<string, string> { ["style"] = "font-weight: 600; color: #333; font-family: monospace; font-size: 14px" }, "useState"),
                            new VElement("div", "1.3.1.2.1.2", new Dictionary<string, string> { ["style"] = "font-size: 12px; color: #666; margin-top: 4px" }, "Manage component state with instant updates and template prediction")
                        }),
                        new VElement("button", "1.3.1.2.2", new Dictionary<string, string> { ["style"] = "padding: 12px 16px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; text-align: left; transition: all 0.2s", ["onclick"] = "Handle3", ["onmouseenter"] = "@client:Handle4", ["onmouseleave"] = "@client:Handle5" }, new VNode[]
                        {
                            new VElement("div", "1.3.1.2.2.1", new Dictionary<string, string> { ["style"] = "font-weight: 600; color: #333; font-family: monospace; font-size: 14px" }, "useEffect"),
                            new VElement("div", "1.3.1.2.2.2", new Dictionary<string, string> { ["style"] = "font-size: 12px; color: #666; margin-top: 4px" }, "Run side effects after component renders (timers, subscriptions, etc.)")
                        }),
                        new VElement("button", "1.3.1.2.3", new Dictionary<string, string> { ["style"] = "padding: 12px 16px; border: 1px solid #ddd; border-radius: 6px; background: white; cursor: pointer; text-align: left; transition: all 0.2s", ["onclick"] = "Handle6", ["onmouseenter"] = "@client:Handle7", ["onmouseleave"] = "@client:Handle8" }, new VNode[]
                        {
                            new VElement("div", "1.3.1.2.3.1", new Dictionary<string, string> { ["style"] = "font-weight: 600; color: #333; font-family: monospace; font-size: 14px" }, "useRef"),
                            new VElement("div", "1.3.1.2.3.2", new Dictionary<string, string> { ["style"] = "font-size: 12px; color: #666; margin-top: 4px" }, "Create mutable refs that persist across renders without triggering updates")
                        })
                    })
                })
            }), (new MObject(activeExample)) ? new VElement("div", "1.4.1", new Dictionary<string, string> { ["style"] = "position: fixed; top: 0px; left: 0px; right: 0px; bottom: 0px; background-color: rgba(0, 0, 0, 0.8); display: flex; align-items: center; justify-content: center; z-index: 1000px", ["onclick"] = "Handle9" }, new VNode[]
            {
                new VElement("div", "1.4.1.1", new Dictionary<string, string> { ["ref"] = "modalRef", ["tabIndex"] = $"{-1}", ["style"] = "background-color: white; padding: 30px; border-radius: 8px; max-width: 90%; max-height: 90%; overflow: auto; position: relative; outline: none", ["onclick"] = "@client:Handle10" }, new VNode[]
                {
                    new VElement("button", "1.4.1.1.1", new Dictionary<string, string> { ["style"] = "position: absolute; top: 10px; right: 10px; padding: 5px 10px; border: none; background: #f0f0f0; border-radius: 4px; cursor: pointer", ["onclick"] = "Handle11" }, "Close"),
                    new VElement("h2", "1.4.1.1.2", new Dictionary<string, string>(), $"Active Example:{(activeExample)}"),
                    new VElement("p", "1.4.1.1.3", new Dictionary<string, string> { ["style"] = "color: #666" }, new VNode[]
                    {
                        new VText("This is where the example component would render.\n              Check the source file in", "1.4.1.1.3.1"),
                        new VElement("code", "1.4.1.1.3.2", new Dictionary<string, string>(), "Pages/Examples/"),
                        new VText("for the full implementation.", "1.4.1.1.3.3")
                    })
                })
            }) : new VNull("1.4"));
    }

    [OnStateChanged("activeExample")]
    private void Effect_0()
    {
        if (activeExample) {
    Console.WriteLine($"Opened example: {activeExample}");
}
    }

    [OnMounted]
    private void Effect_1()
    {
        SetState(nameof(viewCount), viewCount + 1);
        Console.WriteLine("Index page mounted");
    }

    [OnStateChanged("activeExample")]
    private void Effect_2()
    {
        if ((activeExample) != null ? (modalRef) : new VNull("")) {
    modalRef.focus();
}
    }

    private void Effect_3()
    {
        timerRef = DateTimeOffset.Now.ToUnixTimeMilliseconds();
    }

    public void Handle0()
    {
        SetState(nameof(activeExample), "useState");
    }

    public void Handle3()
    {
        SetState(nameof(activeExample), "useEffect");
    }

    public void Handle6()
    {
        SetState(nameof(activeExample), "useRef");
    }

    public void Handle9()
    {
        SetState(nameof(activeExample), new VNull(""));
    }

    public void Handle11()
    {
        SetState(nameof(activeExample), new VNull(""));
    }

    protected override Dictionary<string, string> GetClientHandlers()
    {
        return new Dictionary<string, string>
        {
            ["Handle1"] = @"e => {\n  e.currentTarget.style.borderColor = '#4CAF50';\n  e.currentTarget.style.boxShadow = '0 2px 8px rgba(76, 175, 80, 0.2)';\n}",
            ["Handle2"] = @"e => {\n  e.currentTarget.style.borderColor = '#ddd';\n  e.currentTarget.style.boxShadow = 'none';\n}",
            ["Handle4"] = @"e => {\n  e.currentTarget.style.borderColor = '#4CAF50';\n  e.currentTarget.style.boxShadow = '0 2px 8px rgba(76, 175, 80, 0.2)';\n}",
            ["Handle5"] = @"e => {\n  e.currentTarget.style.borderColor = '#ddd';\n  e.currentTarget.style.boxShadow = 'none';\n}",
            ["Handle7"] = @"e => {\n  e.currentTarget.style.borderColor = '#4CAF50';\n  e.currentTarget.style.boxShadow = '0 2px 8px rgba(76, 175, 80, 0.2)';\n}",
            ["Handle8"] = @"e => {\n  e.currentTarget.style.borderColor = '#ddd';\n  e.currentTarget.style.boxShadow = 'none';\n}",
            ["Handle10"] = @"e => e.stopPropagation()"
        };
    }
}
