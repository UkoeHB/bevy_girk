//! Information access
//!
//! - A set of [InfoAccessConstraint]s may be collected to control access to some information.
//! - An [InfoAccessRights] may be built to define what constraints the rights-holder satisfies.
//! - A set of constraints is satisfied by a set of access rights IFF all constraints are individually satisfied.
//!

//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn targeting_intersection(client_rights: &Option<ClientId>, constraints: &Vec<ClientId>) -> bool
{
    // check if there are any client constraints
    if constraints.len() == 0 { return true; }

    // check if there are any client access rights
    let Some(client_access) = client_rights else { return false; };

    // check if access rights interesect with client constraints
    if constraints.contains(&client_access) { return true; }

    return false;
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub enum InfoAccessConstraint
{
    /// Targets specific clients
    Targets(Vec<ClientId>),
    /// Is global data
    Global,
}

//-------------------------------------------------------------------------------------------------------------------

//todo: replace this with bevy_replicon rooms
#[derive(Component, Clone, Default, Debug)]
pub struct InfoAccessRights
{
    /// Can access targeted client data
    pub client: Option<ClientId>,
    /// Can access global data
    pub global: bool,
}

impl InfoAccessRights
{
    pub fn can_access(&self, access_constraints: &Vec<InfoAccessConstraint>) -> bool
    {
        for constraint in access_constraints.iter()
        {
            let constraint_check = 
                match constraint
                {
                    InfoAccessConstraint::Targets(clients) => targeting_intersection(&self.client, clients),
                    InfoAccessConstraint::Global           => self.global,
                };
            if !constraint_check { return false; }
        }

        return true;
    }
}

//-------------------------------------------------------------------------------------------------------------------
