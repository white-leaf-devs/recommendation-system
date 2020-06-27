-- Your SQL goes here
CREATE FUNCTION update_mean_on_new() RETURNS TRIGGER AS
$BODY$
BEGIN
    INSERT INTO means(user_id, val, score_number)
    VALUES (new.user_id, new.score, 1)
    ON CONFLICT (user_id) DO UPDATE 
        SET val = ((means.val * means.score_number) + excluded.val) / (means.score_number + 1), score_number = means.score_number
        + 1;

    RETURN new;
END;
$BODY$
LANGUAGE plpgsql;

-- Create the trigger on insert
CREATE TRIGGER update_means_on_new_rating
AFTER INSERT ON ratings
FOR EACH ROW
EXECUTE FUNCTION update_mean_on_new();
