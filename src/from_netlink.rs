/// This module contains function to return data gleaned from netlink
pub mod from_netlink {
    use std::{error::Error, net::IpAddr};
    
    use neli::{
        consts::{nl::*, rtnl::*, socket::*},
        err::NlError,
        nl::{NlPayload, Nlmsghdr},
        rtnl::*,
        socket::*,
        types::RtBuffer,
    };


	fn parse_route_table(rtm: Nlmsghdr<NlTypeWrapper, Rtmsg>) -> Result<(), NlError> {
	    let payload = rtm.get_payload()?;
	    // This sample is only interested in the main table.
	    if payload.rtm_table == RtTable::Main {
		let mut src = None;
		let mut dst = None;
		let mut gateway = None;

		for attr in payload.rtattrs.iter() {
		    fn to_addr(b: &[u8]) -> Option<IpAddr> {
		        use std::convert::TryFrom;
		        if let Ok(tup) = <&[u8; 4]>::try_from(b) {
		            Some(IpAddr::from(*tup))
		        } else if let Ok(tup) = <&[u8; 16]>::try_from(b) {
		            Some(IpAddr::from(*tup))
		        } else {
		            None
		        }
		    }

		    match attr.rta_type {
		        Rta::Dst => dst = to_addr(attr.rta_payload.as_ref()),
		        Rta::Prefsrc => src = to_addr(attr.rta_payload.as_ref()),
		        Rta::Gateway => gateway = to_addr(attr.rta_payload.as_ref()),
		        _ => (),
		    }
		}

		if let Some(dst) = dst {
		    print!("{}/{} ", dst, payload.rtm_dst_len);
		} else {
		    print!("default ");
		    if let Some(gateway) = gateway {
		        print!("via {} ", gateway);
		    }
		}

		if payload.rtm_scope != RtScope::Universe {
		    print!(
		        " proto {:?}  scope {:?} ",
		        payload.rtm_protocol, payload.rtm_scope
		    )
		}
		if let Some(src) = src {
		    print!(" src {} ", src);
		}
		println!();
	    }

	    Ok(())
	}


        /// This sample is a simple imitation of the `ip route` command, to demonstrate interaction
    /// with the rtnetlink subsystem.
    pub fn get_route_table() -> Result<(), Box<dyn Error>> {
        let mut socket = NlSocketHandle::connect(NlFamily::Route, None, &[]).unwrap();

        let rtmsg = Rtmsg {
            rtm_family: RtAddrFamily::Inet,
            rtm_dst_len: 0,
            rtm_src_len: 0,
            rtm_tos: 0,
            rtm_table: RtTable::Unspec,
            rtm_protocol: Rtprot::Unspec,
            rtm_scope: RtScope::Universe,
            rtm_type: Rtn::Unspec,
            rtm_flags: RtmFFlags::empty(),
            rtattrs: RtBuffer::new(),
        };
        let nlhdr = {
            let len = None;
            let nl_type = Rtm::Getroute;
            let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Dump]);
            let seq = None;
            let pid = None;
            let payload = rtmsg;
            Nlmsghdr::new(len, nl_type, flags, seq, pid, NlPayload::Payload(payload))
        };
        socket.send(nlhdr).unwrap();

        for rtm_result in socket.iter(false) {
            let rtm = rtm_result?;
            if let NlTypeWrapper::Rtm(_) = rtm.nl_type {
                parse_route_table(rtm)?;
            }
        }
        Ok(())
    }
}
