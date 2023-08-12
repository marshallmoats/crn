/// The molecules play rock paper scissors. The winner transforms the loser into a copy of itself.
pub const ROCK_PAPER_SCISSORS: &str = "
    r=50;
    p=50;
    s=50;
    r+p->2p;
    p+s->2s;
    s+r->2r;
    ";
/// A is the prey and B is the predator.
pub const PREDATOR_PREY: &str = "
    a=100;
    b=100;
    a+b->2b:0.005;
    a->2a;
    b->;
    ";
/// Polya's urn. Draw a marble from the urn, then put two marbles of the same color back in.
pub const POLYA: &str = "
    A = 1;
    B = 1;
    A -> 2A;
    B -> 2B;
    ";
/// Same as the rock paper scissors CRN, but with two more players.
pub const RPSLS: &str = "
    a = 100;
    b = 100;
    c = 100;
    d = 100;
    e = 100;
    a+b->2a;
    b+c->2b;
    c+d->2c;
    d+e->2d;
    e+a->2e;
    a+d->2a;
    b+e->2b;
    c+a->2c;
    d+b->2d;
    e+c->2e;
    ";
/// Determines which of A and B is more abundant.
pub const MAJORITY: &str = "
    A = 30;
    B = 20;
    2A + B -> 3A;
    A + 2B -> 3B;
    ";
/// The majority CRN, but with catalysts that transform into one another.
pub const MAJORITY_CATALYZED: &str = "
    A = 5120;
    B = 4880;
    C = 100;
    D = 100;
    2A + B + C -> 3A + C;
    A + 2B + D -> 3B + D;
    C -> D : 1000000000;
    D -> C : 1000000000;
    ";
/// Approximately calculates the product of A and B. A deterministic simulation will approach it asymptotically.
pub const MULTIPLY: &str = "
    A = 30;
    B = 20;
    C = 0;
    A + B -> A + B + C;
    C ->;
    ";
/// Calculates the product with some random perturbations of catalysts.
pub const MULTIPLY_CATALYZED: &str = "
    A = 30;
    B = 20;
    C = 0;
    D = 5;
    E = 5;
    A + B + D -> A + B + C + D;
    C + E -> E;
    D -> E : 1000000000;
    E -> D : 1000000000;
    ";
/// A basic CRN with two reactions that reach equilibrium.
pub const EQUILIBRIUM: &str = "
    A = 10000;
    B = 10000;
    C = 10000;
    D = 10000;
    A + 2B -> 4C + 3D;
    4C + 3D -> A + 2B;
    ";
/// Looks cool.
pub const CHAIN: &str = "
    A = 100;
    A -> B;
    B -> C;
    C -> D;
    D -> E;
    E -> F;
    F -> G;
    G -> H;
    H -> I;
    I -> J;
    J -> K;
    K -> L;
    ";
/// Honestly, I don't remember what this is supposed to do and I don't remember where I found it.
pub const OTHER: &str = "
    a=50;
    b=40;
    c=100;
    ga=1;
    gb=1;
    gc=1;
    gob=1;
    goc=1;
    goa=1;
    a+b->2b;
    b+c->2c;
    c+a->2a;
    a+gb->iab;
    iab+gob->2b;
    b+lb->gb+lgb;
    gb+lgb->b+lb;
    ibc+goc->2c;
    c+lc->gc+lgc;
    gc+lgc->c+lc;
    c+ga->ica;
    ica+goa->2a;
    a+la->ga+lga;
    ga+lga->a+la;
    ->ga:0.00001;
    ->gb:0.00001;
    ->gc:0.00001;
    ->gob:0.00001;
    ->goc:0.00001;
    ->goa:0.00001;
    ";
