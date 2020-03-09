use super::StarWars;
use async_graphql::Context;

#[async_graphql::Enum(desc = "One of the films in the Star Wars Trilogy")]
#[allow(non_camel_case_types)]
pub enum Episode {
    #[item(desc = "Released in 1977.")]
    NEWHOPE,

    #[item(desc = "Released in 1980.")]
    EMPIRE,

    #[item(desc = "Released in 1983.")]
    JEDI,
}

pub struct Human(usize);

#[async_graphql::Object(desc = "A humanoid creature in the Star Wars universe.")]
impl Human {
    #[field(desc = "The id of the human.")]
    async fn id<'a>(&self, ctx: &Context<'_>) -> &str {
        ctx.data::<StarWars>().chars[self.0].id
    }

    #[field(desc = "The name of the human.")]
    async fn name(&self, ctx: &Context<'_>) -> &str {
        ctx.data::<StarWars>().chars[self.0].name
    }

    #[field(desc = "The friends of the human, or an empty list if they have none.")]
    async fn friends(&self, ctx: &Context<'_>) -> Vec<Character> {
        ctx.data::<StarWars>().chars[self.0]
            .friends
            .iter()
            .map(|id| Human(*id).into())
            .collect()
    }

    #[field(name = "appearsIn", desc = "Which movies they appear in.")]
    async fn appears_in<'a>(&self, ctx: &'a Context<'_>) -> &'a [Episode] {
        &ctx.data::<StarWars>().chars[self.0].appears_in
    }

    #[field(
        name = "homePlanet",
        desc = "The home planet of the human, or null if unknown."
    )]
    async fn home_planet<'a>(&self, ctx: &'a Context<'_>) -> &'a Option<&'a str> {
        &ctx.data::<StarWars>().chars[self.0].home_planet
    }
}

pub struct Droid(usize);

#[async_graphql::Object(desc = "A mechanical creature in the Star Wars universe.")]
impl Droid {
    #[field(desc = "The id of the droid.")]
    async fn id(&self, ctx: &Context<'_>) -> &str {
        ctx.data::<StarWars>().chars[self.0].id
    }

    #[field(desc = "The name of the droid.")]
    async fn name(&self, ctx: &Context<'_>) -> &str {
        ctx.data::<StarWars>().chars[self.0].name
    }

    #[field(desc = "The friends of the droid, or an empty list if they have none.")]
    async fn friends(&self, ctx: &Context<'_>) -> Vec<Character> {
        ctx.data::<StarWars>().chars[self.0]
            .friends
            .iter()
            .map(|id| Droid(*id).into())
            .collect()
    }

    #[field(name = "appearsIn", desc = "Which movies they appear in.")]
    async fn appears_in<'a>(&self, ctx: &'a Context<'_>) -> &'a [Episode] {
        &ctx.data::<StarWars>().chars[self.0].appears_in
    }

    #[field(name = "primaryFunction", desc = "The primary function of the droid.")]
    async fn primary_function<'a>(&self, ctx: &'a Context<'_>) -> &'a Option<&'a str> {
        &ctx.data::<StarWars>().chars[self.0].primary_function
    }
}

pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    #[field]
    async fn hero(
        &self,
        ctx: &Context<'_>,
        #[arg(
            desc = "If omitted, returns the hero of the whole saga. If provided, returns the hero of that particular episode."
        )]
        episode: Episode,
    ) -> Character {
        if episode == Episode::EMPIRE {
            Human(ctx.data::<StarWars>().luke).into()
        } else {
            Droid(ctx.data::<StarWars>().artoo).into()
        }
    }

    #[field]
    async fn human(
        &self,
        ctx: &Context<'_>,
        #[arg(desc = "id of the human")] id: String,
    ) -> Option<Human> {
        ctx.data::<StarWars>().human(&id).map(|id| Human(id))
    }

    #[field]
    async fn droid(
        &self,
        ctx: &Context<'_>,
        #[arg(desc = "id of the droid")] id: String,
    ) -> Option<Droid> {
        ctx.data::<StarWars>().droid(&id).map(|id| Droid(id))
    }
}

#[async_graphql::Interface(
    field(name = "id", type = "&str", context),
    field(name = "name", type = "&str", context),
    field(name = "friends", type = "Vec<Character>", context),
    field(
        name = "appearsIn",
        method = "appears_in",
        type = "&'ctx [Episode]",
        context
    )
)]
pub struct Character(Human, Droid);
