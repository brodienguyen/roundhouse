//! `form_with model:` over an STI base on the emitted CRuby tree. Rails'
//! `polymorphic_path` names a form's action from the record's CLASS, so
//! an edited `Articles::Featured` PATCHes `/articles/featureds/:id`, not
//! `/articles/:id`. Rows hydrate base-classed, so which subclass a
//! record is comes from its `type` column at render time.

use super::emit_and_run;

/// The edit form of a subclass row posts to the subclass's route, a
/// plain row keeps the base's, and a new record of the subclass posts to
/// the subclass's collection. Campfire's `rooms/layouts/_form` posted
/// every room to `/rooms` this way.
#[test]
fn an_sti_records_form_posts_to_its_subclass_route() {
    emit_and_run::real_blog()
        .edit(
            "db/schema.rb",
            "    t.string \"title\"\n    t.text \"body\"\n",
            "    t.string \"title\"\n    t.string \"type\"\n    t.text \"body\"\n",
        )
        .write("app/models/articles/featured.rb", "class Articles::Featured < Article\nend\n")
        .edit(
            "config/routes.rb",
            "  resources :articles do\n",
            "  namespace :articles do\n    resources :featureds, only: [:new, :create, :update]\n  end\n\n  resources :articles do\n",
        )
        .write(
            "app/controllers/articles/featureds_controller.rb",
            r#"class Articles::FeaturedsController < ApplicationController
  def new
    @article = Articles::Featured.new
    render "articles/new"
  end

  def create
    head :no_content
  end

  def update
    head :no_content
  end
end
"#,
        )
        .write(
            "test/controllers/form_with_sti_controller_test.rb",
            r#"require "test_helper"

class FormWithStiControllerTest < ActionDispatch::IntegrationTest
  BODY = "A sufficiently long article body."

  test "an edited subclass record posts to the subclass member route" do
    featured = Articles::Featured.create!(title: "Featured", body: BODY)
    # The id, not the record: a subclass-typed record passed to the
    # base's route helper is not grounded to its id yet.
    get edit_article_url(featured.id)
    assert_response :success
    assert_includes response.body, %(action="/articles/featureds/#{featured.id}")
  end

  test "an edited plain record keeps the base member route" do
    plain = Article.create!(title: "Plain", body: BODY)
    get edit_article_url(plain)
    assert_response :success
    assert_includes response.body, %(action="/articles/#{plain.id}")
  end

  test "a new subclass record posts to the subclass collection route" do
    get new_articles_featured_url
    assert_response :success
    assert_includes response.body, %(action="/articles/featureds")
  end

  test "a new plain record keeps the base collection route" do
    get new_article_url
    assert_response :success
    assert_includes response.body, %(action="/articles")
    refute_includes response.body, %(action="/articles/featureds")
  end
end
"#,
        )
        .run_test("test/controllers/form_with_sti_controller_test.rb")
        .assert_passes();
}
