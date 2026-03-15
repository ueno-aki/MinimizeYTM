pub fn toast_show(
    title: &str,
    artist: &str,
    image_path: &str,
    app_id: &str,
) -> Result<(), windows::core::Error> {
    use windows::{Data::Xml::Dom::XmlDocument, UI::Notifications::ToastNotificationManager};
    let xml: String = format!(
        r#"<toast>
            <audio silent="true"/>
            <visual>
                <binding template="ToastImageAndText02">
                    <image id="1" src="{}" hint-crop="circle"/>
                    <text id="1"> {} </text>
                    <text id="2"> {} </text>
                </binding>
            </visual>
        </toast>"#,
        image_path, title, artist
    );
    let xml_doc = XmlDocument::new()?;
    xml_doc.LoadXml(&xml.into())?;

    let notification =
        windows::UI::Notifications::ToastNotification::CreateToastNotification(&xml_doc)?;
    let toast = ToastNotificationManager::CreateToastNotifierWithId(&app_id.into())?;
    toast.Show(&notification)?;
    Ok(())
}
