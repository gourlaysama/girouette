// This code was autogenerated with `dbus-codegen-rust -c nonblock -m None -s -d org.freedesktop.GeoClue2 -p /org/freedesktop/GeoClue2/Client/1 -i org.freedesktop.DBus.`, see https://github.com/diwic/dbus-rs
#[allow(unused_imports)]
use dbus::arg;
use dbus::nonblock;

pub trait Properties {
    fn get(
        &self,
        interface_name: &str,
        property_name: &str,
    ) -> nonblock::MethodReply<arg::Variant<Box<dyn arg::RefArg + 'static>>>;
    fn get_all(&self, interface_name: &str) -> nonblock::MethodReply<arg::PropMap>;
    fn set(
        &self,
        interface_name: &str,
        property_name: &str,
        value: arg::Variant<Box<dyn arg::RefArg>>,
    ) -> nonblock::MethodReply<()>;
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> Properties
    for nonblock::Proxy<'a, C>
{
    fn get(
        &self,
        interface_name: &str,
        property_name: &str,
    ) -> nonblock::MethodReply<arg::Variant<Box<dyn arg::RefArg + 'static>>> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "Get",
            (interface_name, property_name),
        )
        .and_then(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| Ok(r.0))
    }

    fn get_all(&self, interface_name: &str) -> nonblock::MethodReply<arg::PropMap> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "GetAll",
            (interface_name,),
        )
        .and_then(|r: (arg::PropMap,)| Ok(r.0))
    }

    fn set(
        &self,
        interface_name: &str,
        property_name: &str,
        value: arg::Variant<Box<dyn arg::RefArg>>,
    ) -> nonblock::MethodReply<()> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "Set",
            (interface_name, property_name, value),
        )
    }
}

#[derive(Debug)]
pub struct PropertiesPropertiesChanged {
    pub interface_name: String,
    pub changed_properties: arg::PropMap,
    pub invalidated_properties: Vec<String>,
}

impl arg::AppendAll for PropertiesPropertiesChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.interface_name, i);
        arg::RefArg::append(&self.changed_properties, i);
        arg::RefArg::append(&self.invalidated_properties, i);
    }
}

impl arg::ReadAll for PropertiesPropertiesChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(PropertiesPropertiesChanged {
            interface_name: i.read()?,
            changed_properties: i.read()?,
            invalidated_properties: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for PropertiesPropertiesChanged {
    const NAME: &'static str = "PropertiesChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
}

pub trait Introspectable {
    fn introspect(&self) -> nonblock::MethodReply<String>;
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> Introspectable
    for nonblock::Proxy<'a, C>
{
    fn introspect(&self) -> nonblock::MethodReply<String> {
        self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
            .and_then(|r: (String,)| Ok(r.0))
    }
}

pub trait Peer {
    fn ping(&self) -> nonblock::MethodReply<()>;
    fn get_machine_id(&self) -> nonblock::MethodReply<String>;
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> Peer
    for nonblock::Proxy<'a, C>
{
    fn ping(&self) -> nonblock::MethodReply<()> {
        self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
    }

    fn get_machine_id(&self) -> nonblock::MethodReply<String> {
        self.method_call("org.freedesktop.DBus.Peer", "GetMachineId", ())
            .and_then(|r: (String,)| Ok(r.0))
    }
}

pub trait OrgFreedesktopGeoClue2Client {
    fn start(&self) -> nonblock::MethodReply<()>;
    fn stop(&self) -> nonblock::MethodReply<()>;
    fn location(&self) -> nonblock::MethodReply<dbus::Path<'static>>;
    fn distance_threshold(&self) -> nonblock::MethodReply<u32>;
    fn set_distance_threshold(&self, value: u32) -> nonblock::MethodReply<()>;
    fn time_threshold(&self) -> nonblock::MethodReply<u32>;
    fn set_time_threshold(&self, value: u32) -> nonblock::MethodReply<()>;
    fn desktop_id(&self) -> nonblock::MethodReply<String>;
    fn set_desktop_id(&self, value: String) -> nonblock::MethodReply<()>;
    fn requested_accuracy_level(&self) -> nonblock::MethodReply<u32>;
    fn set_requested_accuracy_level(&self, value: u32) -> nonblock::MethodReply<()>;
    fn active(&self) -> nonblock::MethodReply<bool>;
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> OrgFreedesktopGeoClue2Client
    for nonblock::Proxy<'a, C>
{
    fn start(&self) -> nonblock::MethodReply<()> {
        self.method_call("org.freedesktop.GeoClue2.Client", "Start", ())
    }

    fn stop(&self) -> nonblock::MethodReply<()> {
        self.method_call("org.freedesktop.GeoClue2.Client", "Stop", ())
    }

    fn location(&self) -> nonblock::MethodReply<dbus::Path<'static>> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.GeoClue2.Client",
            "Location",
        )
    }

    fn distance_threshold(&self) -> nonblock::MethodReply<u32> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.GeoClue2.Client",
            "DistanceThreshold",
        )
    }

    fn time_threshold(&self) -> nonblock::MethodReply<u32> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.GeoClue2.Client",
            "TimeThreshold",
        )
    }

    fn desktop_id(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.GeoClue2.Client",
            "DesktopId",
        )
    }

    fn requested_accuracy_level(&self) -> nonblock::MethodReply<u32> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.GeoClue2.Client",
            "RequestedAccuracyLevel",
        )
    }

    fn active(&self) -> nonblock::MethodReply<bool> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.GeoClue2.Client",
            "Active",
        )
    }

    fn set_distance_threshold(&self, value: u32) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.freedesktop.GeoClue2.Client",
            "DistanceThreshold",
            value,
        )
    }

    fn set_time_threshold(&self, value: u32) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.freedesktop.GeoClue2.Client",
            "TimeThreshold",
            value,
        )
    }

    fn set_desktop_id(&self, value: String) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.freedesktop.GeoClue2.Client",
            "DesktopId",
            value,
        )
    }

    fn set_requested_accuracy_level(&self, value: u32) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.freedesktop.GeoClue2.Client",
            "RequestedAccuracyLevel",
            value,
        )
    }
}

#[derive(Debug)]
pub struct OrgFreedesktopGeoClue2ClientLocationUpdated {
    pub old: dbus::Path<'static>,
    pub new: dbus::Path<'static>,
}

impl arg::AppendAll for OrgFreedesktopGeoClue2ClientLocationUpdated {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.old, i);
        arg::RefArg::append(&self.new, i);
    }
}

impl arg::ReadAll for OrgFreedesktopGeoClue2ClientLocationUpdated {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopGeoClue2ClientLocationUpdated {
            old: i.read()?,
            new: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopGeoClue2ClientLocationUpdated {
    const NAME: &'static str = "LocationUpdated";
    const INTERFACE: &'static str = "org.freedesktop.GeoClue2.Client";
}
