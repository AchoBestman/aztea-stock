use validator::Validate;

#[derive(Validate)]
pub struct RoleValidationSchema<'a> {
    #[validate(length(min = 2, max = 50, message = "Le nom doit comporter entre 2 et 50 caractères."))]
    pub name: &'a str,
    #[validate(length(max = 255, message = "La description ne doit pas dépasser 255 caractères."))]
    pub description: Option<&'a str>,
}
