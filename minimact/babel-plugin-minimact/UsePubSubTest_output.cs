using Minimact.AspNetCore.Core;
using Minimact.AspNetCore.Extensions;
using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace Minimact.Components;

[Component]
public partial class UsePubSubTest : MinimactComponent
{
    // usePub: publish
    private string publish_channel = "notifications";

    // useSub: notifications
    private string notifications_channel = "notifications";
    private dynamic notifications_value = null;

    protected override VNode Render()
    {
        StateManager.SyncMembersToState(this);

        var handlePublish = null;

        return new VElement("div", new Dictionary<string, string>(), new VNode[]
        {
            new VElement("h1", new Dictionary<string, string>(), "Pub/Sub Test"),
            new VElement("button", new Dictionary<string, string> { ["onclick"] = "handlePublish" }, "Publish Message"),
            new VElement("div", new Dictionary<string, string>(), new VNode[]
            {
                new VText("Last message:"),
                new VText($"{null}")
            }),
            new VElement("div", new Dictionary<string, string>(), new VNode[]
            {
                new VText("From:"),
                new VText($"{notifications.source}")
            })
        });
    }

    // Publish to publish_channel
    private void publish(dynamic value, PubSubOptions? options = null)
    {
        EventAggregator.Instance.Publish(publish_channel, value, options);
    }

    // Subscribe to notifications_channel
    protected override void OnInitialized()
    {
        base.OnInitialized();
        
        // Subscribe to notifications_channel
        EventAggregator.Instance.Subscribe(notifications_channel, (msg) => {
            notifications_value = msg.Value;
            SetState("notifications_value", notifications_value);
        });
    }
}

