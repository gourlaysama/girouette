// This code was autogenerated with `dbus-codegen-rust -c nonblock -m None -s -d org.freedesktop.GeoClue2 -p /org/freedesktop/GeoClue2/Client/2/Location/0 -i org.freedesktop.DBus.`, see https://github.com/diwic/dbus-rs
use dbus;
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

pub trait OrgFreedesktopGeoClue2Location {
    fn latitude(&self) -> nonblock::MethodReply<f64>;
    fn longitude(&self) -> nonblock::MethodReply<f64>;
    fn accuracy(&self) -> nonblock::MethodReply<f64>;
    fn altitude(&self) -> nonblock::MethodReply<f64>;
    fn speed(&self) -> nonblock::MethodReply<f64>;
    fn heading(&self) -> nonblock::MethodReply<f64>;
    fn description(&self) -> nonblock::MethodReply<String>;
    fn timestamp(&self) -> nonblock::MethodReply<(u64, u64)>;
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>>
    OrgFreedesktopGeoClue2Location for nonblock::Proxy<'a, C>
{
    fn latitude(&self) -> nonblock::MethodReply<f64> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Latitude",
        )
    }

    fn longitude(&self) -> nonblock::MethodReply<f64> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Longitude",
        )
    }

    fn accuracy(&self) -> nonblock::MethodReply<f64> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Accuracy",
        )
    }

    fn altitude(&self) -> nonblock::MethodReply<f64> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Altitude",
        )
    }

    fn speed(&self) -> nonblock::MethodReply<f64> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Speed",
        )
    }

    fn heading(&self) -> nonblock::MethodReply<f64> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Heading",
        )
    }

    fn description(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Description",
        )
    }

    fn timestamp(&self) -> nonblock::MethodReply<(u64, u64)> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.freedesktop.GeoClue2.Location",
            "Timestamp",
        )
    }
}
