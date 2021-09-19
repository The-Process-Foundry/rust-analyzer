use super::*;

pub(super) fn opt_generic_arg_list(p: &mut Parser, colon_colon_required: bool) {
    let m;
    if p.at(T![::]) && p.nth(2) == T![<] {
        m = p.start();
        p.bump(T![::]);
        p.bump(T![<]);
    } else if !colon_colon_required && p.at(T![<]) && p.nth(1) != T![=] {
        m = p.start();
        p.bump(T![<]);
    } else {
        return;
    }

    while !p.at(EOF) && !p.at(T![>]) {
        generic_arg(p);
        if !p.at(T![>]) && !p.expect(T![,]) {
            break;
        }
    }
    p.expect(T![>]);
    m.complete(p, GENERIC_ARG_LIST);
}

// test type_arg
// type A = B<'static, i32, 1, { 2 }, Item=u64, true, false>;
fn generic_arg(p: &mut Parser) {
    match p.current() {
        LIFETIME_IDENT => lifetime_arg(p),
        // test associated_type_bounds
        // fn print_all<T: Iterator<Item, Item::Item, Item::<true>, Item: Display, Item<'a> = Item>>(printables: T) {}
        IDENT if [T![<], T![=], T![:]].contains(&p.nth(1)) => {
            let m = p.start();
            name_ref(p);
            opt_generic_arg_list(p, false);
            match p.current() {
                // NameRef<...> =
                T![=] => {
                    p.bump_any();
                    types::type_(p);
                    m.complete(p, ASSOC_TYPE_ARG);
                }
                // NameRef<...>:
                T![:] if !p.at(T![::]) => {
                    generic_params::bounds(p);
                    m.complete(p, ASSOC_TYPE_ARG);
                }
                _ => {
                    let m = m.complete(p, PATH_SEGMENT).precede(p).complete(p, PATH);
                    let m = paths::type_path_for_qualifier(p, m);
                    m.precede(p).complete(p, PATH_TYPE).precede(p).complete(p, TYPE_ARG);
                }
            }
        }
        // test const_generic_negated_literal
        // fn f() { S::<-1> }
        T!['{'] | T![true] | T![false] | T![-] => const_arg(p),
        k if k.is_literal() => const_arg(p),
        _ => type_arg(p),
    }
}

fn lifetime_arg(p: &mut Parser) {
    let m = p.start();
    lifetime(p);
    m.complete(p, LIFETIME_ARG);
}

pub(super) fn const_arg(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        T!['{'] => {
            expressions::block_expr(p);
            m.complete(p, CONST_ARG);
        }
        k if k.is_literal() => {
            expressions::literal(p);
            m.complete(p, CONST_ARG);
        }
        T![true] | T![false] => {
            expressions::literal(p);
            m.complete(p, CONST_ARG);
        }
        T![-] => {
            let lm = p.start();
            p.bump(T![-]);
            expressions::literal(p);
            lm.complete(p, PREFIX_EXPR);
            m.complete(p, CONST_ARG);
        }
        _ => {
            let lm = p.start();
            paths::use_path(p);
            lm.complete(p, PATH_EXPR);
            m.complete(p, CONST_ARG);
        }
    }
}

fn type_arg(p: &mut Parser) {
    let m = p.start();
    types::type_(p);
    m.complete(p, TYPE_ARG);
}