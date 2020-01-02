"""empty message

Revision ID: fc6358c6a954
Revises: 7a447ff002de
Create Date: 2019-12-31 23:33:45.854616

"""
from alembic import op
import sqlalchemy as sa
import sqlalchemy_utils
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision = 'fc6358c6a954'
down_revision = '7a447ff002de'
branch_labels = None
depends_on = None


def upgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.create_table(
        'program',
        sa.Column(
            'id',
            postgresql.UUID(),
            nullable=False,
            server_default=sa.text("gen_random_uuid()")),
        sa.Column(
            'ast', postgresql.JSON(astext_type=sa.Text()), nullable=False),
        sa.Column('user_id', postgresql.UUID, nullable=False),
        sa.ForeignKeyConstraint(
            ['user_id'],
            ['user.id'],
        ), sa.PrimaryKeyConstraint('id'))
    # ### end Alembic commands ###


def downgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_table('program')
    # ### end Alembic commands ###