using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Timeline("AnimatedCounter_Timeline", 5000, Repeat = true, Easing = "ease-in-out")]
[TimelineKeyframe(0, "count", 0, Label = "start")]
[TimelineKeyframe(0, "color", "blue", Label = "start")]
[TimelineKeyframe(0, "opacity", 1, Label = "start")]
[TimelineKeyframe(1250, "count", 25)]
[TimelineKeyframe(1250, "color", "green")]
[TimelineKeyframe(1250, "opacity", 0.8)]
[TimelineKeyframe(2500, "count", 50, Label = "midpoint")]
[TimelineKeyframe(2500, "color", "red", Label = "midpoint")]
[TimelineKeyframe(2500, "opacity", 0.6, Label = "midpoint")]
[TimelineKeyframe(3750, "count", 75)]
[TimelineKeyframe(3750, "color", "purple")]
[TimelineKeyframe(3750, "opacity", 0.8)]
[TimelineKeyframe(5000, "count", 100, Label = "end")]
[TimelineKeyframe(5000, "color", "blue", Label = "end")]
[TimelineKeyframe(5000, "opacity", 1, Label = "end")]
[TimelineStateBinding("count", Interpolate = true)]
[TimelineStateBinding("color")]
[TimelineStateBinding("opacity", Interpolate = true)]
[Component]
public partial class AnimatedCounter : MinimactComponent
{
    [State]
    private int count = 0;

    [State]
    private string color = "blue";

    [State]
    private int opacity = 1;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        return new VElement("div", "1", new Dictionary<string, string>(), new VNode[]
        {
            new VElement("h1", "1.1", new Dictionary<string, string> { ["style"] = "color: color; opacity: opacity" }, $"Count:{(count)}"),
            new VElement("div", "1.2", new Dictionary<string, string>(), new VNode[]
            {
                new VElement("button", "1.2.1", new Dictionary<string, string> { ["onclick"] = "Handle0" }, "Pause"),
                new VElement("button", "1.2.2", new Dictionary<string, string> { ["onclick"] = "Handle2" }, "Play"),
                new VElement("button", "1.2.3", new Dictionary<string, string> { ["onclick"] = "Handle4" }, "Stop"),
                new VElement("button", "1.2.4", new Dictionary<string, string> { ["onclick"] = "Handle6" }, "Jump to Midpoint")
            })
        });
    }

    public void Handle0()
    {
        timeline.pause();
    }

    public void Handle2()
    {
        timeline.play();
    }

    public void Handle4()
    {
        timeline.stop();
    }

    public void Handle6()
    {
        timeline.seek(2500);
    }

    /// <summary>
    /// Returns JavaScript event handlers for client-side execution
    /// These execute in the browser with bound hook context
    /// </summary>
    protected override Dictionary<string, string> GetClientHandlers()
    {
        return new Dictionary<string, string>
        {
            ["Handle0"] = @"function () {\n  timeline.pause();\n}",
            ["Handle2"] = @"function () {\n  timeline.play();\n}",
            ["Handle4"] = @"function () {\n  timeline.stop();\n}",
            ["Handle6"] = @"function () {\n  timeline.seek(2500);\n}"
        };
    }
}
