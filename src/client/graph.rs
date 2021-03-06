use crate::activities::ActivitiesRequest;
use crate::attachments::AttachmentRequest;
use crate::calendar::CalendarRequest;
use crate::contacts::ContactsRequest;
use crate::drive::DriveRequest;
use crate::education::{EducationMeRequest, EducationRequest, EducationUsersRequest};
use crate::groups::{
    GroupConversationPostRequest, GroupConversationRequest, GroupThreadPostRequest,
};
use crate::http::{
    AsyncHttpClient, BlockingHttpClient, GraphResponse, IntoResponse, RequestClient,
};
use crate::mail::MailRequest;
use crate::onenote::OnenoteRequest;
use crate::planner::PlannerRequest;
use crate::types::{
    boolresponse::BoolResponse, collection::Collection, content::Content, delta::DeltaRequest,
};
use crate::url::GraphUrl;
use crate::{GRAPH_URL, GRAPH_URL_BETA};
use graph_error::GraphFailure;
use graph_oauth::oauth::{AccessToken, OAuth};
use handlebars::*;
use reqwest::header::{HeaderValue, ACCEPT};
use reqwest::Method;
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Ident {
    Me,
    Drives,
    Sites,
    Groups,
    Users,
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        match self {
            Ident::Me => "me",
            Ident::Drives => "drives",
            Ident::Sites => "sites",
            Ident::Groups => "groups",
            Ident::Users => "users",
        }
    }
}

impl Default for Ident {
    fn default() -> Self {
        Ident::Me
    }
}

pub struct Graph<Client> {
    request: Client,
}

impl<'a, Client> Graph<Client>
where
    Client: crate::http::RequestClient,
{
    pub fn v1(&'a self) -> Identify<'a, Client> {
        self.request.set_url(GraphUrl::from_str(GRAPH_URL).unwrap());
        Identify { client: &self }
    }

    /// Use the Graph beta API
    pub fn beta(&'a self) -> Identify<'a, Client> {
        self.request
            .set_url(GraphUrl::from_str(GRAPH_URL_BETA).unwrap());
        Identify { client: &self }
    }

    /// Check if the current host is v1.0.
    pub fn is_v1(&self) -> bool {
        self.request.url().as_str().starts_with(GRAPH_URL)
    }

    /// Check if the current host is beta.
    pub fn is_beta(&self) -> bool {
        self.request.url().as_str().starts_with(GRAPH_URL_BETA)
    }

    pub fn ident(&self) -> Ident {
        self.request.ident()
    }

    /// Set the access token used for requests.
    pub fn set_token(&self, token: &str) {
        self.request.set_token(token);
    }

    pub(crate) fn request(&self) -> &Client {
        &self.request
    }
}

impl<Client> Debug for Graph<Client>
where
    Client: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.request.fmt(f)
    }
}

type GraphBlocking = Graph<BlockingHttpClient>;
type GraphAsync = Graph<AsyncHttpClient>;

impl<'a> GraphBlocking {
    /// Create a new client with an access token.
    ///
    /// # Example
    /// ```
    /// use graph_rs::client::Graph;
    ///
    /// let client = Graph::new("ACCESS_TOKEN");
    /// ```
    /// ```rust,ignore
    /// // Use the v1.0 API
    /// let drive_items: serde_json::Value = client
    ///     .v1()
    ///     .me()
    ///     .drive()
    ///     .root_children()
    ///     .json()?;
    /// ```
    pub fn new(token: &str) -> GraphBlocking {
        let request = BlockingHttpClient::new(GraphUrl::from_str(GRAPH_URL).unwrap());
        request.set_token(token);
        Graph { request }
    }

    pub fn url_ref<F>(&self, f: F)
    where
        F: Fn(&GraphUrl),
    {
        self.request.inner_url_ref(f)
    }
}

impl From<&str> for GraphBlocking {
    fn from(token: &str) -> Self {
        Graph::new(token)
    }
}

impl From<String> for GraphBlocking {
    fn from(token: String) -> Self {
        Graph::new(token.as_str())
    }
}

impl From<&AccessToken> for GraphBlocking {
    fn from(token: &AccessToken) -> Self {
        Graph::new(token.bearer_token())
    }
}

impl TryFrom<&OAuth> for GraphBlocking {
    type Error = GraphFailure;

    fn try_from(oauth: &OAuth) -> Result<Self, Self::Error> {
        let access_token = oauth.get_access_token()?;
        Ok(Graph::from(&access_token))
    }
}

impl<'a> GraphAsync {
    /// Create a new client with an access token.
    ///
    /// # Example
    /// ```
    /// use graph_rs::client::Graph;
    ///
    /// let client = Graph::new_async("ACCESS_TOKEN");
    /// ```
    /// ```rust,ignore
    /// // Use the v1.0 API
    /// let drive_items: serde_json::Value = client
    ///     .v1()
    ///     .me()
    ///     .drive()
    ///     .root_children()
    ///     .json()?;
    /// ```
    pub fn new_async(token: &str) -> GraphAsync {
        let request = AsyncHttpClient::new(GraphUrl::parse(GRAPH_URL).unwrap());
        request.set_token(token);
        Graph { request }
    }

    pub fn url_ref<F>(&self, f: F)
    where
        F: Fn(&GraphUrl) + Sync,
    {
        self.request.url_ref(f)
    }
}

impl From<&str> for GraphAsync {
    fn from(token: &str) -> Self {
        Graph::new_async(token)
    }
}

impl From<String> for GraphAsync {
    fn from(token: String) -> Self {
        Graph::new_async(token.as_str())
    }
}

impl From<&AccessToken> for GraphAsync {
    fn from(token: &AccessToken) -> Self {
        Graph::new_async(token.bearer_token())
    }
}

impl TryFrom<&OAuth> for GraphAsync {
    type Error = GraphFailure;

    fn try_from(oauth: &OAuth) -> Result<Self, Self::Error> {
        let access_token = oauth.get_access_token()?;
        Ok(Graph::from(&access_token))
    }
}

pub struct Identify<'a, Client> {
    client: &'a Graph<Client>,
}

impl<'a, Client> Identify<'a, Client>
where
    Client: crate::http::RequestClient,
{
    /// Select the me endpoint.
    pub fn me(&self) -> IdentMe<'a, Client> {
        self.client.request.set_ident(Ident::Me);
        IdentMe::new("", self.client)
    }

    /// Select the drives endpoint.
    pub fn drives<S: AsRef<str>>(&self, id: S) -> IdentDrives<'a, Client> {
        self.client.request.set_ident(Ident::Drives);
        IdentDrives::new(id.as_ref(), self.client)
    }

    /// Select the sites endpoint.
    pub fn sites<S: AsRef<str>>(&self, id: S) -> IdentSites<'a, Client> {
        self.client.request.set_ident(Ident::Sites);
        IdentSites::new(id.as_ref(), self.client)
    }

    /// Select the groups endpoint.
    pub fn groups<S: AsRef<str>>(&self, id: S) -> IdentGroups<'a, Client> {
        self.client.request.set_ident(Ident::Groups);
        IdentGroups::new(id.as_ref(), self.client)
    }

    /// Select the group lifecycle policies endpoint.
    pub fn group_lifecycle_policies<S: AsRef<str>>(
        &self,
        id: S,
    ) -> GroupLifecyclePolicyRequest<'a, Client> {
        GroupLifecyclePolicyRequest::new(id.as_ref(), self.client)
    }

    /// Select the users endpoint.
    pub fn users<S: AsRef<str>>(&self, id: S) -> IdentUsers<'a, Client> {
        self.client.request.set_ident(Ident::Users);
        IdentUsers::new(id.as_ref(), self.client)
    }

    /// Select the planner endpoint.
    pub fn planner(&self) -> PlannerRequest<'a, Client> {
        PlannerRequest::new(self.client)
    }

    pub fn education(&self) -> EducationRequest<'a, Client> {
        EducationRequest::new(self.client)
    }

    pub fn batch<B: serde::Serialize>(
        &self,
        batch: &B,
    ) -> IntoResponse<'a, DeltaRequest<serde_json::Value>, Client> {
        let client = self.client.request();
        client.set_method(Method::POST);
        client.header(ACCEPT, HeaderValue::from_static("application/json"));
        let body = serde_json::to_string(batch).map_err(GraphFailure::from);
        if let Err(err) = body {
            return IntoResponse::new_error(self.client, err);
        } else if let Ok(body) = body {
            client.set_body(body);
        }
        render_path!(self.client, "$batch", &serde_json::json!({}));
        IntoResponse::new(self.client)
    }
}

register_ident_client!(IdentMe,);
register_ident_client!(IdentDrives,);
register_ident_client!(IdentSites,);
register_ident_client!(IdentGroups,);
register_ident_client!(IdentUsers,);

impl<'a, Client> IdentMe<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( get, serde_json::Value => "me" );
    get!( list_events, Collection<serde_json::Value> => "me/events" );
    get!( settings, serde_json::Value => "me/settings" );
    get!(list_planner_tasks, Collection<serde_json::Value> => "me/planner/tasks");
    patch!( [ update_settings, serde_json::Value => "me/settings" ] );

    pub fn activities(&'a self) -> ActivitiesRequest<'a, Client> {
        self.set_path();
        ActivitiesRequest::new(self.client)
    }

    pub fn education(&self) -> EducationMeRequest<'a, Client> {
        EducationMeRequest::new(&self.client)
    }
}

impl<'a, Client> IdentDrives<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( get, serde_json::Value => "drive/{{RID}}" );
}

impl<'a, Client> IdentSites<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( get, serde_json::Value => "sites/{{RID}}" );
    get!( list_subsites, Collection<serde_json::Value> => "sites/{{RID}}/sites" );
    get!( root, serde_json::Value => "sites/root" );
    get!( | root_tenant, serde_json::Value => "sites/{{id}}" );
    get!( analytics, serde_json::Value => "sites/{{RID}}/analytics" );
    get!( | item_analytics, serde_json::Value => "sites/{{RID}}/items/{{id}}/analytics" );
    get!( | list_item_versions, serde_json::Value => "sites/{{RID}}/items/{{id}}/versions" );

    pub fn lists(&'a self) -> SiteListRequest<'a, Client> {
        SiteListRequest::new(self.client)
    }

    pub fn activities_by_interval(
        &'a self,
        start: &str,
        end: Option<&str>,
        interval: &str,
    ) -> IntoResponse<'a, serde_json::Value, Client> {
        self.client.request().set_method(Method::GET);
        if let Some(end) = end {
            render_path!(self.client, &format!(
                "sites/{{{{RID}}}}/getActivitiesByInterval(startDateTime='{}',endDateTime='{}',interval='{}')",
                start,
                end,
                interval
            ));
        } else {
            render_path!(
                self.client,
                &format!(
                    "sites/{{{{RID}}}}/getActivitiesByInterval(startDateTime='{}',interval='{}')",
                    start, interval
                )
            );
        }
        IntoResponse::new(self.client)
    }
}

register_client!(SiteListRequest,);

impl<'a, Client> SiteListRequest<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( list, Collection<serde_json::Value> => "sites/{{RID}}/lists" );
    get!( | get, serde_json::Value => "sites/{{RID}}/lists/{{id}}" );
    post!( [ create, serde_json::Value => "sites/{{RID}}/lists" ] );

    pub fn items(&'a self) -> SiteListItemRequest<'a, Client> {
        SiteListItemRequest::new(self.client)
    }
}

register_client!(SiteListItemRequest,);

impl<'a, Client> SiteListItemRequest<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( | list, Collection<serde_json::Value> => "sites/{{RID}}/lists/{{id}}/items" );
    get!( || get, serde_json::Value => "sites/{{RID}}/lists/{{id}}/items/{{id2}}" );
    get!( || analytics, serde_json::Value => "sites/{{RID}}/lists/{{id}}/items/{{id2}}/analytics" );
    get!( || list_versions, serde_json::Value => "sites/{{RID}}/lists/{{id}}/items/{{id2}}/versions" );
    post!( [ | create, serde_json::Value => "sites/{{RID}}/lists/{{id}}/items" ] );
    patch!( [ || update, serde_json::Value => "sites/{{RID}}/lists/{{id}}/items/{{id2}}" ] );
    patch!( [ || update_columns, serde_json::Value => "sites/{{RID}}/lists/{{id}}/items/{{id2}}/fields" ] );
    delete!( || delete, GraphResponse<Content> => "sites/{{RID}}/lists/{{id}}/items/{{id2}}" );
}

impl<'a, Client> IdentGroups<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( list, Collection<serde_json::Value> => "groups" );
    get!( get, serde_json::Value => "groups/{{RID}}" );
    get!( delta, DeltaRequest<Collection<serde_json::Value>> => "groups/delta" );
    get!( list_events, Collection<serde_json::Value> => "groups/{{RID}}/events" );
    get!( list_lifecycle_policies, Collection<serde_json::Value> => "groups/{{RID}}/groupLifecyclePolicies" );
    get!( list_member_of, Collection<serde_json::Value> => "groups/{{RID}}/memberOf" );
    get!( list_transitive_member_of, Collection<serde_json::Value> => "groups/{{RID}}/transitiveMemberOf" );
    get!( list_members, Collection<serde_json::Value> => "groups/{{RID}}/members"  );
    get!( list_transitive_members, Collection<serde_json::Value> => "groups/{{RID}}/transitiveMembers" );
    get!( list_owners, Collection<serde_json::Value> => "groups/{{RID}}/owners" );
    get!( list_photos, Collection<serde_json::Value> => "groups/{{RID}}/photos" );
    get!( root_site, Collection<serde_json::Value> => "groups/{{RID}}/sites/root" );
    get!( list_planner_plans, Collection<serde_json::Value> => "groups/{{RID}}/planner/plans" );
    post!( [ create, serde_json::Value => "groups" ] );
    post!( add_favorite, GraphResponse<Content> => "groups/{{RID}}/addFavorite" );
    post!( [ add_member, GraphResponse<Content> => "groups/{{RID}}/members/$ref" ] );
    post!( [ add_owner, GraphResponse<Content> => "groups/{{RID}}/owners/$ref" ] );
    post!( [ check_member_groups, Collection<String> => "groups/{{RID}}/checkMemberGroups" ] );
    post!( [ member_groups, Collection<String> => "groups/{{RID}}/getMemberGroups" ] );
    post!( [ member_objects, Collection<String> => "groups/{{RID}}/getMemberObjects" ] );
    post!( remove_favorite, GraphResponse<Content> => "groups/{{RID}}/removeFavorite" );
    post!( renew, GraphResponse<Content> => "groups/{{RID}}/renew" );
    post!( reset_unseen_count, GraphResponse<Content> => "groups/{{RID}}/resetUnseenCount" );
    post!( subscribe_by_mail, GraphResponse<Content> => "groups/{{RID}}/subscribeByMail" );
    post!( unsubscribe_by_mail, GraphResponse<Content> => "groups/{{RID}}/unsubscribeByMail" );
    post!( [ validate_properties, GraphResponse<Content> => "groups/{{RID}}/validateProperties" ] );
    patch!( [ update, serde_json::Value => "groups/{{RID}}" ] );
    delete!( delete, GraphResponse<Content> => "groups/{{RID}}" );
    delete!( | remove_member, GraphResponse<Content> => "groups/{{RID}}/members/{{id}}/$ref" );
    delete!( | remove_owner, GraphResponse<Content> => "groups/{{RID}}/owners/{{id}}/$ref" );

    pub fn conversations(&self) -> GroupConversationRequest<'a, Client> {
        GroupConversationRequest::new(self.client)
    }

    pub fn conversation_posts(&'a self) -> GroupConversationPostRequest<'a, Client> {
        GroupConversationPostRequest::new(self.client)
    }

    pub fn thread_posts(&'a self) -> GroupThreadPostRequest<'a, Client> {
        GroupThreadPostRequest::new(self.client)
    }
}

impl<'a, Client> IdentUsers<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( get, serde_json::Value => "users/{{RID}}" );
    get!( settings, serde_json::Value => "users/{{RID}}/settings" );
    get!( list, Collection<serde_json::Value> => "users" );
    get!( list_events, Collection<serde_json::Value> => "users/{{RID}}/events" );
    get!( delta, DeltaRequest<Collection<serde_json::Value>> => "users" );
    get!( | list_joined_group_photos, Collection<serde_json::Value> => "users/{{RID}}/joinedGroups/{{id}}/photos" );
    get!( list_planner_tasks, Collection<serde_json::Value> => "users/{{RID}}/planner/tasks");
    post!( [ create, serde_json::Value => "users" ] );
    patch!( [ update, GraphResponse<Content> => "users/{{RID}}" ] );
    patch!( [ update_settings, serde_json::Value => "users/{{RID}}/settings" ] );
    delete!( delete, GraphResponse<Content> => "users/{{RID}}" );

    pub fn activities(&'a self) -> ActivitiesRequest<'a, Client> {
        self.set_path();
        ActivitiesRequest::new(self.client)
    }

    pub fn education(&self) -> EducationUsersRequest<'a, Client> {
        EducationUsersRequest::new(self.client)
    }
}

register_ident_client!(
    GroupLifecyclePolicyRequest,
    glp => "groupLifecyclePolicies",
    ()
);

impl<'a, Client> GroupLifecyclePolicyRequest<'a, Client>
where
    Client: crate::http::RequestClient,
{
    get!( list, Collection<serde_json::Value> => "{{glp}}" );
    get!( get, Collection<serde_json::Value> => "{{glp}}/{{RID}}" );
    post!( [ create, serde_json::Value => "{{glp}}" ] );
    post!( [ add_group, BoolResponse => "{{glp}}/{{RID}}/addGroup" ] );
    post!( [ remove_group, BoolResponse =>  "{{glp}}/{{RID}}/removeGroup" ] );
    patch!( [ update, serde_json::Value => "{{glp}}/{{RID}}" ] );
    patch!( delete, GraphResponse<Content> => "{{glp}}/{{RID}}" );
}
