Agent Island vendors this crate to force macOS foreground notifications to present as banners.

The upstream `NotificationCenterDelegate` did not implement `shouldPresentNotification`, so notifications sent while Agent Island was foreground could be delivered only to Notification Center. The local patch returns `YES` from that delegate method.
